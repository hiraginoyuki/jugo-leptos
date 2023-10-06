#[component]
pub fn App() -> impl IntoView {
    let puzzle = create_rw_signal(SeedablePuzzle::<usize>::new((4, 4)));
    let shape = create_memo(move |_| with!(|puzzle| puzzle.shape()));
    let seed = create_memo(move |_| with!(|puzzle| *puzzle.seed()));
    let seed_formatted = create_memo(move |_| {
        BASE64_URL_SAFE
            .encode(seed())
            .chars()
            .chunks(11)
            .into_iter()
            .map(|chunk| chunk.collect::<String>())
            .join("\n")
    });
    let pieces_sorted = create_memo(move |_| {
        with!(|puzzle| {
            let (width, height) = puzzle.shape();
            puzzle.iter_indexed().fold(
                vec![Default::default(); width * height].into_boxed_slice(),
                |mut pieces, (idx2, &piece)| {
                    pieces[piece] = idx2;
                    pieces
                },
            )
        })
    });

    let history = create_rw_signal(String::new());
    let dev_mode = create_rw_signal(false);
    let game_state = create_rw_signal(GameState::NotSolving);

    let timer_secs_ref = create_node_ref::<Div>();
    let timer_millis_ref = create_node_ref::<Div>();
    let input_ref = create_node_ref::<Input>();

    let slide = move |idx| {
        let moved = match puzzle.update_if_some(move |p| p.slide_from(idx)) {
            Some(moved @ 1..) => moved,
            _ => return 0,
        };

        #[rustfmt::skip]
        request_animation_frame(move || return_with_try! {
            input_ref.get_untracked()?.set_scroll_left(i32::MAX);
        });

        game_state.update_guarded(|mut state| match *state {
            GameState::NotSolving => {
                *state = GameState::Solving {
                    since: Instant::now(),
                };
            }
            GameState::Solving { since } if puzzle.with_untracked(|puzzle| puzzle.is_solved()) => {
                *state = GameState::Solved {
                    took: since.elapsed(),
                };
            }
            _ => {}
        });

        moved
    };

    let on_keydown = move |event: KeyboardEvent| {
        let key = event.key();

        match key.as_ref() {
            " " => {
                puzzle.update(|p| *p = SeedablePuzzle::new(p.shape()));
                history.update(|history| history.clear());
                game_state.set(GameState::NotSolving);
            }

            "D" => dev_mode.update(|dev_mode| *dev_mode = !*dev_mode),
            "1" => game_state.set(GameState::NotSolving),
            "2" => game_state.set(GameState::Solving {
                since: Instant::now(),
            }),
            "3" => {
                if let Some(since) = game_state.with(|state| match *state {
                    GameState::Solving { since } => Some(since),
                    _ => None,
                }) {
                    game_state.set(GameState::Solved {
                        took: since.elapsed(),
                    });
                }
            }

            _ => {
                if let Some(&idx) = KEY_IDX_MAP.get(&key) {
                    let moved = slide(idx);

                    if moved > 0 {
                        update!(|history| history.push_str(&key));
                    }
                }
            }
        }
    };

    #[rustfmt::skip]
    let render_piece = move |shape: Memo<(usize, usize)>| move |piece| {
        let (width, _) = shape();
        let index = create_memo(move |_| {
            pieces_sorted.with(move |pieces| pieces[piece])
        });
        let opacity = match piece {
            0 => "opacity-0",
            _ => "",
        };

        view! {
            <div
                class=move || {
                    let (x, y) = index();
                    let ideal_piece = y * width + x + 1;
                    format!(
                        "absolute w-16 h-16 rounded-lg flex justify-center items-center
                        font-mono text-2xl shadow transition-all ease-out-circ duration-[100ms]
                        translate-x-[calc(var(--x)*4.5rem)] translate-y-[calc(var(--y)*4.5rem)]
                        pointer-events-none {opacity} {}",
                        match ideal_piece == piece { // is_solved
                            true => "bg-neutral-100 dark:bg-neutral-200 text-neutral-800",
                            false => "bg-neutral-900 dark:bg-neutral-800 text-neutral-200",
                        },
                    )
                }
                style=("--x", move || index().0)
                style=("--y", move || index().1)
            >
                {piece}
            </div>
        }
    };

    #[rustfmt::skip]
    pre_paint(move || return_with_try! {
        let time = game_state
            .with(|state| state.solve_time())
            .unwrap_or(Duration::ZERO);

        let secs = format!("{:02}", time.as_secs());
        let millis = format!("{:03}", time.subsec_millis());

        timer_secs_ref()?.set_text_content(Some(&secs));
        timer_millis_ref()?.set_text_content(Some(&millis));
    });

    view! {
        <div class="flex h-[100dvh] w-full place-content-evenly">
            <div class=move || format!(
                "flex my-auto justify-center items-start
                ease-out-circ transition-all transform-gpu duration-150 {}",
                match dev_mode() {
                    false => "translate-x-0",
                    true => "-translate-x-16",
                },
            )>
                <div class="flex flex-col">
                    <div class="grid grid-flow-col grid-cols-[1fr_min-content_1fr]
                                child:font-mono child:flex child:items-end">
                        <div class="text-5xl justify-end" _ref=timer_secs_ref>"00"</div>
                        <div class="text-2xl">"."</div>
                        <div class="text-2xl" _ref=timer_millis_ref>"000"</div>
                    </div>
                    <div class="mx-auto my-4 grid grid-cols-4 gap-2">
                        <For
                            each=move || shape.with(|&(w, h)| (0..w * h))
                            key=Clone::clone
                            children=render_piece(shape)
                        />
                        <For
                            each=move || shape.with(|&(w, h)| (0..w * h))
                            key=Clone::clone
                            children=move |index| {
                                let (width, _) = shape();
                                let slide = move |event: Event| {
                                    event.prevent_default();
                                    slide((index % width, index / width));
                                };
                                view! {
                                    <div
                                        class="w-16 h-16"
                                        on:mousedown=move |e| slide(e.into())
                                        on:mousemove=move |e| if e.buttons() & 1 == 1 { slide(e.into()) }
                                        on:touchstart=move |e| slide(e.into())
                                        // on:touchmove=move |e| slide(e.into())
                                    />
                                }
                            }
                        />
                    </div>
                    <input _ref=input_ref
                        type="text"
                        readonly
                        class=move || format!(
                            "mx-auto mb-auto w-[17.5rem] p-2 shadow rounded-md outline-none
                            ring-inset ring-1 focus:ring-2 font-mono bg-neutral-100 dark:bg-neutral-800 
                            transition-all ease-out-circ duration-[40ms] {}",
                            match dev_mode() {
                                false => "ring-neutral-400 dark:ring-neutral-600 focus:ring-violet-400 focus:dark:ring-violet-500",
                                true => "ring-yellow-500 dark:ring-yellow-600 focus:ring-yellow-500 focus:dark:ring-yellow-500",
                            }
                        )
                        on:keydown=on_keydown
                        prop:value=history
                    />
                </div>
                <div class=move || format!(
                    "ease-out-circ transition-all transform-gpu duration-150 {}",
                    match dev_mode() {
                        false => "-translate-x-6 opacity-0",
                        true => "translate-x-0",
                    },
                )>
                    <AnimatedShow when=dev_mode hide_delay=Duration::from_millis(150)>
                        <div class="absolute ml-6 mt-3">
                            <pre class="mb-3">{seed_formatted}</pre>
                            <pre class="text-sm">"is_solved(): "{move || with!(|puzzle| puzzle.is_solved())}</pre>
                            <pre class="text-sm">"game_state: "{move || format!("{:#?}", game_state())}</pre>
                        </div>
                    </AnimatedShow>
                </div>
            </div>
        </div>
    }
}

use std::time::Duration;
use wasm_timer::Instant;

use leptos::{ev::*, html::*, *};

use base64::{prelude::*, Engine};
use derive_more::*;
use itertools::Itertools;
use jugo::{BoxPuzzle, Piece, Puzzle};
use macros::return_with_try;
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

use crate::signal_ext::SignalUpdateConditional;

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

fn pre_paint(callback: impl Clone + Fn() + 'static) {
    request_animation_frame(move || {
        untrack(callback.clone());
        pre_paint(callback); // recurse
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

#[derive(Clone, Debug)]
enum GameState {
    NotSolving,
    Solving { since: Instant },
    Solved { took: Duration },
}

impl GameState {
    pub fn solve_time(&self) -> Option<Duration> {
        match self {
            GameState::Solving { since } => Some(since.elapsed()),
            GameState::Solved { took } => Some(*took),
            _ => None,
        }
    }
}
