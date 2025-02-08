use rand::seq::SliceRandom;
use yew::prelude::*;
use std::collections::HashMap;
use gloo_timers::future::TimeoutFuture;
use wasm_bindgen_futures::spawn_local;

fn solve(mut pieces: Vec<u32>) -> Vec<Vec<u32>> {
    let mapping: HashMap<u32, u32> = pieces.iter().enumerate().map(|(i, &p)| (i as u32, p)).collect();
    let mut circles = vec![];

    // essentially take (really any) element and use
    // its piece value as an index for the next,
    // continue until that piece number has been used (not in the list)

    while let Some(mut piece) = pieces.pop() {
        let mut circle = vec![];
                
        loop {
            circle.push(piece);
            piece = match mapping.get(&piece) {
                Some(&next_piece) => next_piece,
                None => break, // this should never break
            };

            if pieces.contains(&piece) == false { break; }
            pieces.remove(pieces.iter().position(|x| *x == piece).expect("this should never panic"));
        }

        circle.reverse();
        circles.push(circle);
    }

    circles
}

enum AppMessage {
    Shuffle,
    Solve,
    Click(u32),
    Swap(u32, u32),
    Reset,
    Resize(u32),
}

struct App {
    link: ComponentLink<Self>,
    pieces: Vec<u32>,
    size: u32,
    current: Option<u32>
}

impl Component for App {
    type Message = AppMessage;

    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        let size = 20_u32;
        Self {
            current: None,
            link: link,
            size: size,
            pieces: (0..(size * size)).collect(),
        }
    }

    fn update(&mut self, message: Self::Message) -> ShouldRender {
        match message {
            AppMessage::Swap(a, b) => {
                self.pieces.swap(a as usize, b as usize);
                true
            },
            AppMessage::Shuffle => {
                self.pieces.shuffle(&mut rand::rng());
                true
            },
            AppMessage::Click(piece) => {
                match self.current {
                    None => { self.current = Some(piece); true }
                    Some(current) => {
                        self.link.send_message(AppMessage::Swap(current, piece));
                        self.current = None;
                        true
                    }
                }
            },
            AppMessage::Solve => {
                let solution = solve(self.pieces.clone());
                
                let link = self.link.clone();
                spawn_local(async move {
                    for circle in solution {
                        for window in circle.windows(2) {
                            if let [a, b] = window {
                                link.send_message(AppMessage::Swap(*a, *b));
                            }
                            TimeoutFuture::new(400).await;
                        }
                        TimeoutFuture::new(1200).await;
                    }

                });

                false
            },
            AppMessage::Reset => {
                self.pieces = (0..(self.size * self.size)).collect();
                true
            }
            AppMessage::Resize(size) => {
                self.size = size;
                self.link.send_message(AppMessage::Reset);
                true
            }
        }

    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        false
    }

    fn view(&self) -> Html {
        let img = "assets/image.jpeg";
        // divide with 100 so we don't have to multiply by 100 later
        let k = 100f64 / (self.size as f64);

        html! {
            <main class="container mx-auto p-4 bg-gray-200 h-screen flex flex-col content-center space-y-2">
                <section class="h-16 p-2 bg-red-200 flex justify-center flex-row rounded space-x-2">
                    <input type="number" value={format!("{}", self.size)} min="1" max="100" oninput=self.link.callback({
                        let size = self.size;
                        move |event: InputData| {
                            match event.value.parse::<u32>() {
                                Ok(size) => Self::Message::Resize(size),
                                Err(_) => Self::Message::Resize(size),
                            }
                        }
                    })/>
                    <button class="bg-blue-400 rounded" onclick=self.link.callback(|_| Self::Message::Shuffle)>{"Shuffle"}</button>
                    <button class="bg-blue-400 rounded" onclick=self.link.callback(|_| Self::Message::Solve)>{"Solve"}</button>
                    <button class="bg-blue-400 rounded" onclick=self.link.callback(|_| Self::Message::Reset)>{"Reset"}</button>
                </section>
                <section class="bg-purple-400 grow rounded flex flex-row justify-center">
                    <main class="bg-yellow-400 aspect-square relative mx-auto">
                        {for self.pieces.clone().into_iter().enumerate().map(|(index, piece)| {
                            let (x, y) = ((piece / self.size) as f64, (piece % self.size) as f64);
                            let (u, v) = ((index / self.size as usize) as f64, (index % self.size as usize) as f64);

                            html! {
                                <button
                                    onclick=self.link.callback(move |_| Self::Message::Click(index as u32)) 
                                    style={"transition: all 400ms ease-in-out"}
                                    class={
                                        format!("absolute overflow-hidden aspect-square w-[{}%] left-[{:.8}%] top-[{}%] scale-none {}",
                                            100f64 / (self.size as f64),
                                            x * k,
                                            y * k,
                                            if self.current == Some(index as u32) {"scale-[0.80]"} else {""}
                                        )
                                    }>
                                    <img
                                        src={img}
                                        class={
                                            format!("pointer-events-none max-w-[{0}%] w-[{0}%] absolute -left-[{1:.8}%] -top-[{2:.8}%]",
                                                self.size * 100,
                                                u * 100f64,
                                                v * 100f64
                                            )}/>
                                </button>
                            }
                        })}
                    </main>
                </section>
            </main>
        }
    }
}

fn main() {
    yew::start_app::<App>();
}