use std::collections::HashMap;
use std::collections::HashSet;

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
    from: Mrc<Airport>,
    to: Mrc<Airport>,
    duration: Cell<usize>,
    max_duration: usize,
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
    fn goto(&mut self, to: Mrc<Airport>, from: Mrc<Airport>, duration: usize) {
        to.borrow_mut().status = Visited::Visiting;
        self.travels.push(Travel {
            to: to.clone(),
            from: from.clone(),
            duration: Cell::new(duration),
            max_duration: duration,
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
    let speed = 10;
    let (travel, dp) = use_store::<Travels>();
    let _ = use_store::<Timer>();
    if let Some(current) = &travel.travels.get(0) {
        if current.duration.get() <= speed {
            // we arrived
            Dispatch::<Location>::new().reduce_mut(|l| l.goto(current.to.clone()));
            dp.reduce_mut(|t| t.travels.remove(0));
        } else {
            // show flyng position on map
            current.duration.set(current.duration.get() - speed);
            let progress = 1.0 - current.duration.get() as f64 / current.max_duration as f64;
            let from = current.from.borrow();
            let to = current.to.borrow();
            let lat = from.lat() + (to.lat() - from.lat()) * progress;
            let mut long = from.long() + (to.long() - from.long()) * progress;
            // Fixing crossing the date line
            let diff = to.long() - from.long();
            if diff < -180.0 {
                long += 360.0 * progress;
            }
            if diff > 180.0 {
                long -= 360.0 * progress;
            }
            //info!("Position: {}, {}", lat, long);
            eval(format!(
                " document.circle.setLatLng([{},{}]);
                document.map.setView([{}, {}, 5]) ",
                lat, long, lat, long
            ));
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
    fn goto(&mut self, airport: Mrc<Airport>) {
        self.airport = airport;
        let x = self.airport.clone();
        Dispatch::<Stats>::new().reduce(|stats| {
            stats.visit(x);
            stats
        });
        self.airport.borrow_mut().status = Visited::Visited;
        // Dispatch::<Options>::new().set(Default::default());
    }
    fn set_marker(&self) {
        let airport = self.airport.borrow_mut();
    }
}
use std::ops::Deref;
#[function_component(ShowLocation)]
fn show_location() -> html {
    let (loc, _) = use_store::<Location>();
    let stats = Dispatch::<Stats>::new().get();
    //let (_, _) = use_store::<Airports>();
    let airport = &loc.airport.borrow();
    eval(format!(
        "document.map.setView([{},{}], 5);",
        airport.lat, airport.long
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
impl Options {
    fn sort_by_duration(&mut self) {
        self.options.sort_by(|a, b| a.duration.cmp(&b.duration));
        self.options.reverse(); // Show routes with most points first
    }
    fn sort_by_points(&mut self) {
        self.options.sort_by(|a, b| {
            //  a.points.cmp(&b.points))

            let airports = Dispatch::<Airports>::new().get();
            let stats = Dispatch::<Stats>::new().get();

            let a = airports.get(&a.arr_iata);
            let a = a.borrow();
            let b = airports.get(&b.arr_iata);
            let b = b.borrow();
            a.points.cmp(&b.points)
        });
        // self.options.reverse(); // Show routes with most points first
    }
    fn sort_visited(&mut self) {
        self.options.sort_by(|a, b| {
            let airports = Dispatch::<Airports>::new().get();
            let stats = Dispatch::<Stats>::new().get();

            let a = airports.get(&a.arr_iata);
            let a = a.borrow();
            let b = airports.get(&b.arr_iata);
            let b = b.borrow();
            let a_country = stats.visited_country(&a.country_code);
            let b_country = stats.visited_country(&b.country_code);
            a_country.cmp(&b_country)
        })
    }
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
    //    #[serde(default)]
    //    _points: usize,
}
impl Route {
    fn route_points(&mut self) {
        let airports = Dispatch::<Airports>::new().get();
        let dep = airports.get(&self.dep_iata);
        let dep = dep.borrow();
        let arr = airports.get(&self.arr_iata);
        let arr = arr.borrow();
        dep.lowest_neighbor_points
            .set(dep.lowest_neighbor_points.get().min(arr.points.get()));
    }
    // Rate route based on target and duration
    /*
    fn points(&mut self) {
        let airports = Dispatch::<Airports>::new().get();
        let stats = Dispatch::<Stats>::new().get();
        let dep = airports.get(&self.dep_iata);
        let dep = dep.borrow();
        let dep_country = dep.country_code.clone();
        let arr = airports.get(&self.arr_iata);
        let arr = arr.borrow();
        let arr_country = arr.country_code.clone();
        if !stats.visited_country(&arr_country) {
            self.points += 1_000_000
        };
        if arr.status == Visited::NotVisited {
            self.points += 100_000
        };
        if dep_country != arr_country {
            self.points += 10000;
        }
        self.points += 10_000 - self.duration;
    }
    */
}

#[derive(Default, Clone)]
struct Airports {
    airports: HashMap<String, Mrc<Airport>>,
}
impl Store for Airports {
    fn new() -> Self {
        spawn_local(async move {
            let res = query(
                " select iata_code, name, lat,long, country_code
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
            let start = Dispatch::<Airports>::new().get().get(&"HND".to_string());
            Dispatch::<Location>::new().reduce_mut(|loc| loc.goto(start));
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
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default, PartialOrd, Ord)]
enum Visited {
    Visited,
    Visiting,
    #[default]
    NotVisited,
}
/*
impl Ord for Visited {
    fn max(self, other: Self) -> Self
    where
        Self: Sized, {
}
*/
#[derive(Deserialize, PartialEq, Debug)]
struct Airport {
    country_code: String,
    name: String,
    iata_code: String,
    lat: String,
    long: String,
    #[serde(skip)]
    status: Visited,
    #[serde(skip)]
    points: Cell<usize>,
    #[serde(skip)]
    lowest_neighbor_points: Cell<usize>,
}
impl Airport {
    fn lat(&self) -> f64 {
        self.lat.parse::<f64>().unwrap()
    }
    fn long(&self) -> f64 {
        self.long.parse::<f64>().unwrap()
    }
}
impl Default for Airport {
    fn default() -> Self {
        Airport {
            country_code: "GER".to_string(),
            iata_code: "DUS".to_string(),
            name: "Dusseldorf".to_string(),
            lat: "0".into(),
            long: "0".into(),
            status: Default::default(),
            points: 0.into(),
            lowest_neighbor_points: 0.into(),
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
                <Autoclicker/>
            </div>
            <div>
            <ShowStats/>
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
    var circle = L.circle([0,0]).addTo(document.map);
            document.circle = circle;


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
        let mut x: Vec<Route> = JsValue::into_serde(&res).unwrap_or_else(|res| {
            info!("{}", res);
            panic!("Failed unwraping")
        });
        x.iter_mut().for_each(|x| x.route_points());

        airport.points.set(airport.lowest_neighbor_points.get() + 1);
        if x.len() == 0 {
            info!("No way back, giving you an emergence route");
            x.push(Route {
                dep_iata: airport.iata_code.clone(),
                dep_time: "0".into(),
                arr_iata: "FRA".into(),
                duration: 1000,
                // points: 0,
            })
        }
        Dispatch::<Options>::new().set(Options { options: x });
        Dispatch::<Options>::new().reduce_mut(|options| options.sort_by_points());
    });
}
#[function_component(GetOptions)]
fn get_options() -> html {
    let prev = use_state(|| Mrc::new(Airport::default()));
    let (travels, _) = use_store::<Travels>();
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
        let travel = format!(
            "From {} to {} at {},",
            dep_airport.borrow().iata_code, arr_airport.borrow().deref(), option.dep_time        );
        let duration = option.duration;
        let duration_text = format!(", duration: {}m", duration);
        let arr_country = arr_airport.borrow().country_code.clone();
        let country_visited = Dispatch::<Stats>::new().get().visited_country(&arr_country);
        let class = if country_visited  {
            classes!("bg-green-400")
            } else {
            Default::default()
        };
        let country = html! {
            <span {class}> {arr_country}</span>
        };
        let points = arr_airport.borrow().points.get();

        let class = match arr_airport.borrow().status  {
        Visited::Visited =>  classes!("bg-green-400"),
            Visited::Visiting => classes!("bg-yellow-400"),
            Visited::NotVisited => classes!(),
        };
        let onclick = Callback::once(move |_| {
            // Dispatch::<Location>::new().reduce_mut(|loc| loc.goto(arr_airport));
           Dispatch::<Travels>::new().reduce_mut(|loc| loc.goto(arr_airport, dep_airport, duration));
        });
        html! {
            <div>
                <p > <span {class}>  {travel}</span> {country} {duration_text} {", "} {points}
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
#[derive(Default)]
struct Stats {
    visited_airports: Cell<usize>,
    visited_countries: Mrc<HashSet<String>>,
}
impl Store for Stats {
    fn new() -> Self {
        Default::default()
    }
    fn should_notify(&self, _old: &Self) -> bool {
        true
    }
}
impl Stats {
    fn visit(&self, airport: Mrc<Airport>) {
        if airport.borrow().status != Visited::Visited {
            self.visited_airports.set(self.visited_airports.get() + 1);
        }
        let new = self
            .visited_countries
            .borrow_mut()
            .insert(airport.borrow().country_code.clone());
        if new {
            eval(format!(
                "var marker = L.marker([{}, {}]).addTo(document.map); ",
                airport.borrow().lat,
                airport.borrow().long
            ));
            info!("Set marker");
            // let location = Dispatch::<Location>::new().get();
            // location.set_marker();
        }
    }
    fn visited_country(&self, country_code: &str) -> bool {
        self.visited_countries.borrow().contains(country_code)
    }
}
#[function_component(ShowStats)]
fn show_stats() -> html {
    let (stats, _) = use_store::<Stats>();
    let (_, _) = use_store::<Airports>();
    html! {
        <p> {format!("Visited: {} airports, {} countries", stats.visited_airports.get(), stats.visited_countries.borrow().len())}
        </p>
    }
}
use std::cell::RefCell;
#[derive(Default)]
struct AutoClicker {
    // enabled: bool,
    interval: RefCell<Option<Interval>>,
}
impl AutoClicker {
    fn toggle(&self) {
        let mut interval = self.interval.borrow_mut();
        if let Some(_) = interval.as_ref() {
            *interval = None;
        } else {
            *interval = Some(Interval::new(3000, || {
                let route = Dispatch::<Options>::new().get().options[0].clone();
                let airports = Dispatch::<Airports>::new().get();
                Dispatch::<Travels>::new().reduce_mut(|loc| {
                    loc.goto(
                        airports.get(&route.arr_iata),
                        airports.get(&route.dep_iata),
                        route.duration,
                    )
                });
            }))
        }
    }
}
impl Store for AutoClicker {
    fn new() -> Self {
        Self {
            interval: RefCell::new(None),
        }
    }
    fn should_notify(&self, _old: &Self) -> bool {
        false
    }
}
#[function_component(Autoclicker)]
fn autoclicker() -> html {
    let (clicker, dp) = use_store::<AutoClicker>();
    let onclick = Callback::from(move |_| {
        dp.reduce(|c| {
            c.toggle();
            c
        });
    });
    html! {
        <button {onclick}>{"Autoclicker"}</button>
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
