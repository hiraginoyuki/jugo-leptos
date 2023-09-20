use leptos::ev::KeyboardEvent;
use leptos::html::Input;
use leptos::*;

use derive_more::*;
use std::ops::Deref;

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use jugo::{BoxPuzzle, Piece, Puzzle};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

#[rustfmt::skip]
static KEY_IDX_MAP: phf::Map<&'static str, (usize, usize)> = phf::phf_map! {
    "4" => (0, 0), "5" => (1, 0), "6" => (2, 0), "7" => (3, 0),
    "r" => (0, 1), "t" => (1, 1), "y" => (2, 1), "u" => (3, 1),
    "f" => (0, 2), "g" => (1, 2), "h" => (2, 2), "j" => (3, 2),
    "v" => (0, 3), "b" => (1, 3), "n" => (2, 3), "m" => (3, 3),
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
pub fn App(cx: Scope) -> impl IntoView {
    let (puzzle, set_puzzle) = create_signal(cx, SeedablePuzzle::<u8>::new((4, 4)));
    let (history, set_history) = create_signal(cx, String::new());

    let history_ref = create_node_ref::<Input>(cx);
    create_effect(cx, move |_| {
        if let Some(input) = history_ref.get() {
            input.set_value(history.get().as_ref());
        }
    });

    let seed = move || puzzle.with(|puzzle| BASE64_URL_SAFE_NO_PAD.encode(puzzle.seed()));
    let puzzle = move || puzzle.with(|puzzle| format!("{}", puzzle.deref()));
    let on_keydown = move |event: KeyboardEvent| {
        event.prevent_default();

        let key = event.key();

        if let Some(&idx) = KEY_IDX_MAP.get(&key) {
            let d = set_puzzle
                .try_update(move |p| p.slide_from(idx))
                .expect("how would this fail?")
                .unwrap_or(0);
            if d != 0 {
                set_history.update(move |history| {
                    history.push_str(&key);
                });
            }
            return;
        }

        match key.as_ref() {
            " " => {
                set_puzzle.update(|p| *p = SeedablePuzzle::new(p.shape()));
                set_history.update(|history| history.clear());
            }
            _ => {}
        };
    };

    view! { cx,
        <pre> {seed} </pre>
        <pre> {puzzle} </pre>
        <input
            _ref=history_ref
            type="text"
            on:keydown=on_keydown />
    }
}
