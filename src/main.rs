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
use yewdux::mrc::Mrc;

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
            _interval: Interval::new(50, interval),
        }
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
#[derive(Default, Clone)]
struct Travel {
    to: Mrc<Airport>,
    duration: Cell<usize>,
}

impl Display for Travel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Traveling to {}, time left: {}",
            self.to.borrow().deref(),
            self.duration.get()
        )
    }
}
#[derive(Default, Clone)]
struct Travels {
    travels: Vec<Travel>,
}
impl Travels {
    fn goto(&mut self, a: Mrc<Airport>, duration: usize) {
        a.borrow_mut().status = Visited::Visiting;
        self.travels.push(Travel {
            to: a.clone(),
            duration: Cell::new(duration),
        });

        // self.to = Some(a);
        //self.duration = duration.into();
    }
}
impl Store for Travels {
    fn new() -> Self {
        Default::default()
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
#[function_component(Travelling)]
fn travelling() -> html {
    let (travel, dp) = use_store::<Travels>();
    let _ = use_store::<Timer>();
    if let Some(current) = &travel.travels.get(0) {
        if current.duration.get() == 0 {
            // we arrived
            Dispatch::<Location>::new().reduce_mut(|l| l.goto(current.to.clone()));
            dp.reduce_mut(|t| t.travels.remove(0));
        } else {
            current.duration.set(current.duration.get() - 1);
        }
        let html = travel.travels.iter().map(|travel| {
            html! { <p> {travel}</p>}
        });
        html! {
        <div>
            {html.collect::<Vec<_>>()}
        </div>
        }
    } else {
        html! {}
    }
}

#[derive(Clone, Default, PartialEq)]
struct Location {
    airport: Mrc<Airport>,
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
    fn goto(&mut self, a: Mrc<Airport>) {
        a.borrow_mut().status = Visited::Visited;
        self.airport = a;
        // Dispatch::<Options>::new().set(Default::default());
    }
}
use std::ops::Deref;
#[function_component(ShowLocation)]
fn show_location() -> html {
    let (loc, _) = use_store::<Location>();
    //let (_, _) = use_store::<Airports>();
    let airport = &loc.airport.borrow();
    eval(format!(
        "document.map.setView([{},{}], 5);
        var marker = L.marker([{}, {}]).addTo(document.map);
        ",
        airport.lat, airport.long, airport.lat, airport.long,
    ));
    // show_options_for(airport.clone());
    html! {
        <p> {format!("You are in {}", airport.deref())} </p>
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
    airports: HashMap<String, Mrc<Airport>>,
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
                    dp.add(airport);
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
    fn get(&self, iata_code: &String) -> Mrc<Airport> {
        if self.airports.is_empty() {
            // not initialized
            Default::default()
        } else {
            self.airports.get(iata_code).unwrap().clone()
        }
    }
    fn add(&mut self, airport: Airport) {
        let iata_code = airport.iata_code.clone();
        self.airports.insert(iata_code, Mrc::new(airport));
    }
}
use std::fmt::Display;
impl Display for Airport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.iata_code)
    }
}
#[derive(Copy, Clone, PartialEq, Debug, Default)]
enum Visited {
    Visited,
    Visiting,
    #[default]
    NotVisited,
}
#[derive(Deserialize, PartialEq, Debug)]
struct Airport {
    name: String,
    iata_code: String,
    lat: String,
    long: String,
    #[serde(skip)]
    status: Visited,
}
impl Default for Airport {
    fn default() -> Self {
        Airport {
            iata_code: "DUS".to_string(),
            name: "Dusseldorf".to_string(),
            lat: "0".into(),
            long: "0".into(),
            status: Default::default(),
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
fn show_options_for(airport: Mrc<Airport>) {
    spawn_local(async move {
        let airport = airport.borrow();
        // go on, inject yourself...
        let q = format!(
            "
                select dep_iata, arr_iata, duration, min(dep_time) as dep_time 
                from  airlines 
                where dep_iata = '{}'
                group by arr_iata
                ;",
            &airport.iata_code
        );
        info!("Query: {}", q);
        let res = query(q);
        let res = get_from_js(res).await.unwrap();
        let x: Vec<Route> = JsValue::into_serde(&res).unwrap_or_else(|res| {
            info!("{}", res);
            panic!("Failed unwraping")
        });
        Dispatch::<Options>::new().set(Options { options: x });
    });
}
#[function_component(GetOptions)]
fn get_options() -> html {
    let prev = use_state(|| Mrc::new(Airport::default()));
    let (travels, dp) = use_store::<Travels>();
    let _ = use_store::<Location>();
    //let onclick = Callback::from(move |_| {
    // let current = Dispatch::<Location>::new().get();
    let last_travel = travels.travels.last();
    if let Some(travel) = last_travel {
        if *prev != travel.to {
            prev.set(travel.to.clone());
            // info!("To, {}", travel.to.);
            show_options_for(travel.to.clone());
        }
    } else {
        let loc = Dispatch::<Location>::new().get();
        // info!("Empty, {}", loc.airport.borrow());
        show_options_for(loc.airport.clone());
    }

    //});
    /*
    html! {
       <div>
       <button {onclick}>{"Search"}</button>
       </div>
    }
    */
    html! {}
}

#[function_component(ShowOptions)]
fn options() -> html {
    let (options, _) = use_store::<Options>();
    let (airports, _) = use_store::<Airports>();
    let _ = use_store::<Location>();
    // let airports = airports..borrow();
    let options = options.options.iter().map(|option| {
        let dep_airport = airports.get(&option.dep_iata);
        let arr_airport = airports.get(&option.arr_iata);
        let text = format!(
            "From {} to {} at {}, duration {}m",
            dep_airport.borrow().iata_code, arr_airport.borrow().deref(), option.dep_time, option.duration
        );
        let duration = option.duration;

        let class = match arr_airport.borrow().status  {
        Visited::Visited =>  classes!("bg-green-400"),
            Visited::Visiting => classes!("bg-yellow-400"),
            Visited::NotVisited => classes!(),
        };
        let onclick = Callback::once(move |_| {
            // Dispatch::<Location>::new().reduce_mut(|loc| loc.goto(arr_airport));
           Dispatch::<Travels>::new().reduce_mut(|loc| loc.goto(arr_airport, duration));
        });
        html! {
            <div>
                <p {class}> {text}
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
