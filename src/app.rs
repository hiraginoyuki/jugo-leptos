use std::time::Duration;
use wasm_timer::Instant;

use leptos::{ev::KeyboardEvent, html::Input, html::Div};
use leptos::*;

use derive_more::*;
use itertools::Itertools;
use base64::{prelude::*, Engine};
use jugo::{BoxPuzzle, Piece, Puzzle};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;
use macros::return_with_try;

#[rustfmt::skip]
static KEY_IDX_MAP: phf::Map<&'static str, (usize, usize)> = phf::phf_map! {
    "4" => (0, 0), "5" => (1, 0), "6" => (2, 0), "7" => (3, 0),
    "r" => (0, 1), "t" => (1, 1), "y" => (2, 1), "u" => (3, 1),
    "f" => (0, 2), "g" => (1, 2), "h" => (2, 2), "j" => (3, 2),
    "v" => (0, 3), "b" => (1, 3), "n" => (2, 3), "m" => (3, 3),

    // "R" => (0, 1), "T" => (1, 1), "Y" => (2, 1), "U" => (3, 1),
    // "F" => (0, 2), "G" => (1, 2), "H" => (2, 2), "J" => (3, 2),
    // "V" => (0, 3), "B" => (1, 3), "N" => (2, 3), "M" => (3, 3),
};

fn raf_loop(closure: impl Clone + Fn() + 'static) {
    request_animation_frame(move || {
        closure();

        raf_loop(closure);
    });
}

#[derive(Deref, DerefMut)]
struct SeedablePuzzle<T: Piece> {
    #[deref]
    #[deref_mut]
    puzzle: BoxPuzzle<T>,
    seed: [u8; 32],
}

impl<T: Piece> SeedablePuzzle<T> {
    #[inline]
    fn new_from_seed(seed: [u8; 32], shape: (usize, usize)) -> Self {
        Self {
            puzzle: BoxPuzzle::random_with_rng(&mut Xoshiro256StarStar::from_seed(seed), shape),
            seed,
        }
    }
    #[inline]
    fn new(shape: (usize, usize)) -> Self {
        Self::new_from_seed(rand::thread_rng().gen(), shape)
    }

    #[inline]
    fn seed(&self) -> &[u8; 32] {
        &self.seed
    }
}

#[component]
pub fn App() -> impl IntoView {
    let puzzle = create_rw_signal(SeedablePuzzle::<u8>::new((4, 4)));
    let history = create_rw_signal(String::new());
    let dev_mode = create_rw_signal(false);

    let start_time = create_rw_signal(None::<Instant>);
    let solve_time = create_rw_signal(None::<Duration>);
    let timer_secs_ref = create_node_ref::<Div>();
    let timer_millis_ref = create_node_ref::<Div>();

    let input_ref = create_node_ref::<Input>();
    create_effect(move |_| return_with_try! {
        history.with(|_| ()); // hack: track dependency
        input_ref.get()?.set_scroll_left(i32::MAX);
    });

    let seed_formatted = move || {
        puzzle.with(|puzzle| {
            BASE64_URL_SAFE
                .encode(puzzle.seed())
                .chars()
                .chunks(11)
                .into_iter()
                .map(|chunk| chunk.collect::<String>())
                .join("\n")
        })
    };

    let on_keydown = move |event: KeyboardEvent| return_with_try! {
        let key = event.key();

        if let Some(&idx) = KEY_IDX_MAP.get(&key) {
            let moved_for = puzzle
                .try_update(move |p| p.slide_from(idx))?
                .unwrap_or(0);

            if moved_for != 0 {
                history.update(|history| history.push_str(&key));
                start_time.update(|start_time| {
                    match start_time {
                        None => {
                            _ = start_time.insert(Instant::now());
                        }
                        Some(time) if puzzle.with(|p| p.is_solved()) => {
                            solve_time.set(Some(time.elapsed()));
                        }
                        _ => {}
                    };
                });
            }
        }
        
        match key.as_ref() {
            " " => {
                puzzle.update(|p| *p = SeedablePuzzle::new(p.shape()));
                history.update(|history| history.clear());
                start_time.set(None);
                solve_time.set(None);
            }
            "D" => {
                dev_mode.update(|dev_mode| *dev_mode = !*dev_mode);
            }
            "1" => {
                start_time.set(None);
                solve_time.set(None)
            }
            "2" => {
                start_time.set(Some(Instant::now()));
            }
            "3" => {
                solve_time.set(Some(start_time.get()?.elapsed()));
            }
            _ => {}
        }
    };
    let render_piece = |(index, piece)| {
        let solved = index + 1 == piece as usize;
        let colors = match solved {
            true => "bg-neutral-100 dark:bg-neutral-200 text-neutral-800",
            false => "bg-neutral-900 dark:bg-neutral-800 text-neutral-200",
        };
        let visibility = match piece {
            0 => "opacity-0",
            _ => "",
        };

        view! {
            <div class=format!("w-16 h-16 rounded-lg flex justify-center items-center transition-all ease-out duration-75
                                font-mono text-2xl shadow {colors} {visibility}")>
                {piece}
            </div>
        }
    };

    raf_loop(move || return_with_try! {
        let elapsed = solve_time.get_untracked().unwrap_or_else(|| match start_time.get_untracked() {
            Some(start_time) => start_time.elapsed(),
            None => Duration::ZERO,
        });

        let secs = format!("{:02}", elapsed.as_secs());
        let millis = format!("{:03}", elapsed.subsec_millis());

        timer_secs_ref.get_untracked()?.set_text_content(Some(&secs));
        timer_millis_ref.get_untracked()?.set_text_content(Some(&millis));
    });

    view! {
        <div class="flex h-[100dvh] w-full place-content-evenly">
            <div class=move || format!(
                "flex my-auto justify-center items-start
                ease-out transition-all transform-gpu duration-150 {}",
                match dev_mode.get() {
                    false => "translate-x-0",
                    true => "-translate-x-16",
                },
            )>
                <div class="flex flex-col">
                    <div class="grid grid-flow-col grid-cols-[1fr_min-content_1fr]
                                child:font-mono child:flex child:items-end">
                        <div _ref=timer_secs_ref class="text-5xl justify-end">
                            "00"
                        </div>
                        <div class="text-2xl">"."</div>
                        <div _ref=timer_millis_ref class="text-2xl">
                            "000"
                        </div>
                    </div>
                    <div class="mx-auto my-4 grid grid-cols-4 gap-2">
                        <For
                            each=move || puzzle.with(|puzzle| {
                                puzzle
                                    .iter()
                                    .cloned()
                                    .enumerate()
                                    .collect::<Vec<_>>()
                            })
                            key=Clone::clone
                            children=render_piece />
                    </div>
                    <input _ref=input_ref
                        type="text"
                        readonly
                        class=move || format!(
                            "mx-auto mb-auto w-[17.5rem] p-2 shadow rounded-md outline-none
                            ring-inset ring-1 focus:ring-2 font-mono bg-neutral-100 dark:bg-neutral-800 
                            transition-all ease-out duration-[40ms] {}",
                            match dev_mode.get() {
                                false => "ring-neutral-400 dark:ring-neutral-600 focus:ring-violet-400 focus:dark:ring-violet-500",
                                true => "ring-yellow-500 dark:ring-yellow-600 focus:ring-yellow-500 focus:dark:ring-yellow-500",
                            }
                        )
                        on:keydown=on_keydown
                        prop:value=history />
                </div>
                <div class=move || format!(
                    "ease-out transition-all transform-gpu duration-150 {}",
                    match dev_mode.get() {
                        false => "-translate-x-6 opacity-0",
                        true => "translate-x-0",
                    },
                )>
                    <AnimatedShow when=dev_mode hide_delay=Duration::from_millis(150)>
                        <div class="absolute ml-6 mt-3">
                            <pre class="mb-3">
                                {seed_formatted}
                            </pre>
                            <pre class="text-sm">"start_time: "{move || format!("{:?}", start_time.get())}</pre>
                            <pre class="text-sm">"solve_time: "{move || format!("{:?}", solve_time.get())}</pre>
                            <pre class="text-sm">"is_solved(): "{move || puzzle.with(|p| p.is_solved())}</pre>
                        </div>
                    </AnimatedShow>
                </div>
            </div>
        </div>
    }
}
