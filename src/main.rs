use std::collections::HashMap;

use log::info;
use yew::function_component;
use yew::prelude::*;
use yewdux::prelude::*;

use gloo_utils::format::JsValueSerdeExt;
use serde::Deserialize;
use std::cell::Cell;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;

use gloo_timers::callback::Interval;
pub struct Timer {
    _interval: Interval,
}
impl Timer {}
impl Store for Timer {
    fn new() -> Self {
        let interval = || {
            Dispatch::<Timer>::new().reduce(|timer| timer);
        };
        Timer {
            _interval: Interval::new(100, interval),
        }
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
#[derive(Default, Clone)]
struct Travel {
    to: Option<Airport>,
    duration: Cell<usize>,
}
impl Travel {
    fn goto(&mut self, a: Airport, duration: usize) {
        self.to = Some(a);
        self.duration = duration.into();
    }
}
impl Store for Travel {
    fn new() -> Self {
        Default::default()
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
#[function_component(Travelling)]
fn travelling() -> html {
    let (travel, dp) = use_store::<Travel>();
    let _ = use_store::<Timer>();
    if let Some(airport) = &travel.to {
        if travel.duration.get() == 0 {
            // we arrived
            Dispatch::<Location>::new().reduce_mut(|l| l.goto(travel.to.clone().unwrap()));
            dp.reduce_mut(|t| t.to.take());
        } else {
            travel.duration.set(travel.duration.get() - 1);
        }
        html! {
            <p>  {format!("Traveling to {}, time left: {}", airport, travel.duration.get())} </p>
        }
    } else {
        html! {}
    }
}

#[derive(Clone, Default, PartialEq)]
struct Location {
    airport: Airport,
}
impl Store for Location {
    fn new() -> Self {
        let _dp = Dispatch::<Airports>::new().get();
        // TODO chooser component
        Default::default()
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
impl Location {
    fn goto(&mut self, a: Airport) {
        self.airport = a;
        Dispatch::<Options>::new().set(Default::default());
    }
}
#[function_component(ShowLocation)]
fn show_location() -> html {
    let (loc, _) = use_store::<Location>();
    //let (_, _) = use_store::<Airports>();
    let airport = &loc.airport;
    eval(format!(
        "document.map.setView([{},{}], 8);
        var marker = L.marker([{}, {}]).addTo(document.map);
        ",
        airport.lat, airport.long, airport.lat, airport.long,
    ));
    show_options_for(airport.clone());
    html! {
        <p> {format!("You are in {}", airport)} </p>
    }
}
#[derive(Clone, Default, PartialEq)]
struct Options {
    options: Vec<Route>,
}
impl Store for Options {
    fn new() -> Self {
        Default::default()
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    // yew::Renderer::<model::Model>::new().render();
    yew::start_app::<Model>();
}
#[derive(Deserialize, Clone, PartialEq, Debug)]
struct Route {
    //airline_iata: String,
    // airline_icao: String,
    // dep_icao: String,
    dep_iata: String,
    dep_time: String,
    // arr_icao: String,
    arr_iata: String,
    // arr_time: String,
    duration: usize,
}

#[derive(Default, Clone)]
struct Airports {
    airports: HashMap<String, Airport>,
}
impl Store for Airports {
    fn new() -> Self {
        spawn_local(async move {
            let res = query(
                " select iata_code, name, lat,long
                     from airports"
                    .to_string(),
            );
            let res = get_from_js(res).await.unwrap();
            let x: Vec<Airport> = JsValue::into_serde(&res).unwrap();
            // assert!(x.len() == 1);
            Dispatch::<Airports>::new().reduce_mut(move |dp| {
                for airport in x {
                    dp.add(airport.clone());
                }
            });
        });
        Default::default()
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
impl Airports {
    fn get(&self, iata_code: &String) -> Airport {
        if self.airports.is_empty() {
            // not initialized
            Default::default()
        } else {
            self.airports.get(iata_code).unwrap().clone()
        }
    }
    fn add(&mut self, airport: Airport) {
        let iata_code = airport.iata_code.clone();
        self.airports.insert(iata_code, airport);
    }
}
use std::fmt::Display;
impl Display for Airport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.iata_code)
    }
}
#[derive(Deserialize, Clone, PartialEq, Debug)]
struct Airport {
    name: String,
    iata_code: String,
    lat: String,
    long: String,
}
impl Default for Airport {
    fn default() -> Self {
        Airport {
            iata_code: "DUS".to_string(),
            name: "Dusseldorf".to_string(),
            lat: "0".into(),
            long: "0".into(),
        }
    }
}
#[function_component(Model)]
fn model() -> html {
    info!("Starting");

    html! {
        <div class="flex">
            <div>
                <ShowLocation/>
                <div id="map"/>
            </div>
            <div>
            <Travelling/>
            <GetOptions/>
            <ShowOptions/>
            </div>
        <Map/>


        </div>
    }
}
#[function_component(Map)]
fn init_map() -> html {
    eval(
        " var map = L.map('map').setView([51.505, -0.09], 13);
                document.map = map;

    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: '&copy; <a href=\"http://www.openstreetmap.org/copyright\">OpenStreetMap</a>'
    }).addTo(document.map);
                "
        .into(),
    );
    html! {}
}
fn show_options_for(s: Airport) {
    spawn_local(async move {
        // go on, inject yourself...
        let q = format!(
            "
                select dep_iata, arr_iata, duration, min(dep_time) as dep_time 
                from  airlines 
                where dep_iata = '{}'
                group by arr_iata
                ;",
            &s.iata_code
        );
        info!("Query: {}", q);
        let res = query(q);
        let res = get_from_js(res).await.unwrap();
        let x: Vec<Route> = JsValue::into_serde(&res).unwrap();
        Dispatch::<Options>::new().set(Options { options: x });
    });
}
#[function_component(GetOptions)]
fn get_options() -> html {
    let onclick = Callback::from(|_| {
        let current = Dispatch::<Location>::new().get();
        show_options_for(current.airport.clone());
    });
    html! {
       <div>
       <button {onclick}>{"Search"}</button>
       </div>
    }
}

#[function_component(ShowOptions)]
fn options() -> html {
    let (options, _) = use_store::<Options>();
    let (airports, _) = use_store::<Airports>();
    // let airports = airports..borrow();
    let options = options.options.iter().map(|option| {
        let dep_airport = airports.get(&option.dep_iata);
        let arr_airport = airports.get(&option.arr_iata);
        let text = format!(
            "From {} to {} at {}, duration {}m",
            dep_airport.iata_code, arr_airport, option.dep_time, option.duration
        );
        let duration = option.duration;
        let onclick = Callback::once(move |_| {
            // Dispatch::<Location>::new().reduce_mut(|loc| loc.goto(arr_airport));
           Dispatch::<Travel>::new().reduce_mut(|loc| loc.goto(arr_airport, duration));
        });
        html! {
            <div>
                <p> {text}
                <button class="bg-blue-100 hover:bg-blue-300 border-solid  px-4 rounded" {onclick}>{"Travel"} </button>
                </p>
            </div>

        }
    });

    html! { <div class={classes!("float-right")}>
            <h2 class="text-center"> {"Your flight options"} </h2>

        {options.collect::<Vec<_>>()}

        </div>
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window)]
    fn query(query: String) -> JsValue;
    #[wasm_bindgen]
    fn eval(js: String);

}

async fn get_from_js(res: JsValue) -> Result<JsValue, JsValue> {
    let promise = js_sys::Promise::resolve(&res.into());
    let result = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(result)
}
