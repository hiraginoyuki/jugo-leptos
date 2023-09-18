use leptos::*;
use leptos_meta::*;

use derive_more::*;
use std::ops::Deref;

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use jugo::{Puzzle, BoxPuzzle, Piece};
use rand::{SeedableRng, Rng};
use rand_xoshiro::Xoshiro256StarStar;

#[derive(Deref, DerefMut)]
struct PuzzleWrapper<T: Piece> {
    #[deref] #[deref_mut]
    puzzle: BoxPuzzle<T>,
    seed: [u8; 32],
}

impl<T: Piece> PuzzleWrapper<T> {
    fn new_from_rng(rng: &mut impl Rng, shape: (usize, usize)) -> Self {
        let seed = rng.gen();
        let rng = &mut Xoshiro256StarStar::from_seed(seed);

        Self {
            puzzle: BoxPuzzle::random_with_rng(rng, shape),
            seed,
        }
    }
    fn new(shape: (usize, usize)) -> Self {
        Self::new_from_rng(&mut rand::thread_rng(), shape)
    }

    fn gen_from_rng(&mut self, rng: &mut impl Rng) {
        *self = Self::new_from_rng(rng, self.puzzle.shape());
    }
    fn gen(&mut self) {
        self.gen_from_rng(&mut rand::thread_rng());
    }

    fn seed(&self) -> &[u8; 32] {
        &self.seed
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context(cx);

    let (puzzle, slide_from, _, generate) = {
        let puzzle_wrapper = PuzzleWrapper::<u8>::new((4, 4));
        let (puzzle, set_puzzle) = create_signal(cx, puzzle_wrapper);

        (
            puzzle,
            move |idx| set_puzzle.update(move |p| { p.slide_from(idx); }),
            move |dir, dist| set_puzzle.update(move |p| { p.slide_towards(dir, dist); }),
            move || set_puzzle.update(move |p| { p.gen(); }),
        )
    };

    view! {
        cx,

        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/jugo-leptos.css"/>

        <pre>
            // seed
            {move || puzzle.with(|p| BASE64_URL_SAFE_NO_PAD.encode(&p.seed()))}
        </pre>

        <pre>
            // puzzle itself
            {move || puzzle.with(|p| format!("{}", p.deref()))}
        </pre>

        <input on:keydown=move |event| {
            #[rustfmt::skip]
            let idx = match event.key().as_ref() {
                " " => {
                    event.prevent_default();
                    generate();
                    return;
                }
                "4" => (0, 0), "5" => (1, 0), "6" => (2, 0), "7" => (3, 0),
                "r" => (0, 1), "t" => (1, 1), "y" => (2, 1), "u" => (3, 1),
                "f" => (0, 2), "g" => (1, 2), "h" => (2, 2), "j" => (3, 2),
                "v" => (0, 3), "b" => (1, 3), "n" => (2, 3), "m" => (3, 3),
                _ => return,
            };

            slide_from(idx);
        } />
    }
}
