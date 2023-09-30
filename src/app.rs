use leptos::ev::KeyboardEvent;
use leptos::html::Input;
use leptos::*;

use derive_more::*;
use itertools::Itertools;
use std::ops::Deref;

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
    let (puzzle, set_puzzle) = create_signal(SeedablePuzzle::<u8>::new((4, 4)));
    let (history, set_history) = create_signal(String::new());

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
            let moved_for = set_puzzle
                .try_update(move |p| p.slide_from(idx))?
                .unwrap_or(0);

            if moved_for != 0 {
                set_history.update(|history| history.push_str(&key));
            }
        } else if key == " " {
            set_puzzle.update(|p| *p = SeedablePuzzle::new(p.shape()));
            set_history.update(|history| history.clear());
        }
    };

    let render = move |((x, y), piece)| {
        let solved = (x + 1) + (y * 4) == piece as usize;

        let colors = match solved {
            true => "bg-white text-black",
            false => "bg-black text-white",
        };
        let display = match piece {
            0 => "opacity-0",
            _ => "",
        };

        view! {
            <div class=format!("w-16 h-16 flex justify-center items-center rounded-lg font-mono text-2xl {colors} {display}")>
                {piece}
            </div>
        }
    };

    // {move || puzzle.with(|puzzle| {
    //     puzzle
    //         .iter_indexed()
    //         .map(|(idx, &piece)| render((idx, piece)))
    //         .collect_view()
    // })}
    view! {
        <div class="flex h-screen items-center">
            <div class="ml-auto mr-5 grid grid-cols-4 gap-1.5">
                <For
                    each=move || puzzle.with(|puzzle| puzzle
                        .iter_indexed()
                        .map(|(idx, &piece)| (idx, piece))
                        .collect::<Vec<_>>())
                    key=move |&(idx, piece)| (idx, piece)
                    children=move |(idx, piece)| render((idx, piece))
                />
            </div>
            <div class="mr-auto flex flex-col items-center">
                <pre class="mb-5">{seed}</pre>
                <input
                    readonly
                    class="font-mono p-2 w-72 bg-stone-100 dark:bg-stone-800 rounded-md outline-none ring-1 focus:ring-2 ring-gray-300 dark:ring-stone-600 focus:ring-inset focus:ring-violet-400 dark:focus:ring-violet-500"
                    _ref=history_ref
                    type="text"
                    on:keydown=on_keydown />
            </div>
        </div>
    }
}
