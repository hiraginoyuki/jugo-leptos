use leptos::ev::KeyboardEvent;
use leptos::html::Input;
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

    "R" => (0, 1), "T" => (1, 1), "Y" => (2, 1), "U" => (3, 1),
    "F" => (0, 2), "G" => (1, 2), "H" => (2, 2), "J" => (3, 2),
    "V" => (0, 3), "B" => (1, 3), "N" => (2, 3), "M" => (3, 3),
};

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

    let history_ref = create_node_ref::<Input>();
    create_effect(move |_| {
        return_with_try! {
            let element = history_ref.get()?;
            element.set_value(history.get().as_ref());
            element.set_scroll_left(i32::MAX);
        }
    });

    let seed = move || {
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
            }
        }
        
        match key.as_ref() {
            " " => {
                puzzle.update(|p| *p = SeedablePuzzle::new(p.shape()));
                history.update(|history| history.clear());
            }
            "D" => {
                dev_mode.update(|dev_mode| *dev_mode = !*dev_mode);
            }
            _ => {}
        }
    };

    let render_piece = |((x, y), piece)| {
        let solved = (x + 1) + (y * 4) == piece as usize;

        let colors = match solved {
            true => "bg-neutral-100 dark:bg-neutral-200 text-neutral-800",
            false => "bg-neutral-900 dark:bg-neutral-800 text-neutral-200",
        };
        let visibility = match piece {
            0 => "opacity-0",
            _ => "",
        };

        view! {
            <div class=format!("w-16 h-16 rounded-lg flex justify-center items-center font-mono text-2xl shadow {colors} {visibility}")>
                {piece}
            </div>
        }
    };

    view! {
        <div class="flex h-screen w-full place-content-evenly">
            <div class="flex my-auto justify-center items-start">
                <div class="flex flex-col">
                    <div class="mx-auto mt-auto mb-5 grid grid-cols-4 gap-2">
                        <For
                            each=move || puzzle.with(|puzzle| puzzle
                                .iter_indexed()
                                .map(|(idx, &piece)| (idx, piece))
                                .collect::<Vec<_>>())
                            key=Clone::clone
                            children=render_piece
                        />
                    </div>
                    <input
                        readonly
                        class=move || {
                            let ring_colors = match dev_mode.get() {
                                false => "ring-neutral-400 dark:ring-neutral-600 focus:ring-violet-400 focus:dark:ring-violet-500",
                                true => "ring-yellow-500 dark:ring-yellow-600 focus:ring-yellow-500 focus:dark:ring-yellow-500",
                            };

                            format!("mx-auto mb-auto w-[17.5rem] p-2 shadow rounded-md outline-none
                                ring-1 focus:ring-2 focus:ring-inset font-mono bg-neutral-100 dark:bg-neutral-800 {ring_colors}")
                        }
                        _ref=history_ref
                        type="text"
                        on:keydown=on_keydown />
                </div>
                <div class=move || {
                    let opacity = if dev_mode.get() { "opacity-100" } else { "opacity-0" };
                    format!("transition-opacity duration-75 {opacity}")
                }>
                    <pre class="absolute ml-6 mt-3">
                        {seed}
                    </pre>
                </div>
            </div>
        </div>
    }
}
