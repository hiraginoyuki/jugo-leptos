use leptos::*;
use leptos::ev::KeyboardEvent;
use leptos_meta::*;

use derive_more::*;
use std::ops::Deref;

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use jugo::{BoxPuzzle, Piece, Puzzle};
use rand::{Rng, SeedableRng};
use rand_xoshiro::Xoshiro256StarStar;

#[derive(Deref, DerefMut)]
struct PuzzleWrapper<T: Piece> {
    #[deref]
    #[deref_mut]
    puzzle: BoxPuzzle<T>,
    seed: [u8; 32],
}

impl<T: Piece> PuzzleWrapper<T> {
    #[inline]
    fn new_from_rng(rng: &mut impl Rng, shape: (usize, usize)) -> Self {
        let seed = rng.gen();
        let rng = &mut Xoshiro256StarStar::from_seed(seed);

        Self {
            puzzle: BoxPuzzle::random_with_rng(rng, shape),
            seed,
        }
    }
    #[inline]
    fn new(shape: (usize, usize)) -> Self {
        Self::new_from_rng(&mut rand::thread_rng(), shape)
    }

    #[inline]
    fn gen_from_rng(&mut self, rng: &mut impl Rng) {
        *self = Self::new_from_rng(rng, self.puzzle.shape());
    }
    #[inline]
    fn gen(&mut self) {
        self.gen_from_rng(&mut rand::thread_rng());
    }

    #[inline]
    fn seed(&self) -> &[u8; 32] {
        &self.seed
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let (puzzle, slide_from, generate) = {
        let puzzle_wrapper = PuzzleWrapper::<u8>::new((4, 4));
        let (puzzle, set_puzzle) = create_signal(cx, puzzle_wrapper);

        (
            puzzle,
            move |idx| {
                set_puzzle.update(move |p| {
                    p.slide_from(idx);
                })
            },
            move || {
                set_puzzle.update(move |p| {
                    p.gen();
                })
            },
        )
    };

    let seed = move || puzzle.with(|puzzle| BASE64_URL_SAFE_NO_PAD.encode(puzzle.seed()));
    let puzzle = move || puzzle.with(|puzzle| format!("{}", puzzle.deref()));

    let on_keydown = move |event: KeyboardEvent| {
        #[rustfmt::skip]
        let idx = match event.key().as_ref() {
            "4" => (0, 0), "5" => (1, 0), "6" => (2, 0), "7" => (3, 0),
            "r" => (0, 1), "t" => (1, 1), "y" => (2, 1), "u" => (3, 1),
            "f" => (0, 2), "g" => (1, 2), "h" => (2, 2), "j" => (3, 2),
            "v" => (0, 3), "b" => (1, 3), "n" => (2, 3), "m" => (3, 3),

            " " => {
                event.prevent_default();
                generate();
                return;
            }
            _ => return,
        };

        slide_from(idx);
    };

    view! { cx,
        <pre style=""> {seed} </pre>
        <pre style=""> {puzzle} </pre>
        <input style="" on:keydown=on_keydown />
    }
}
