#![allow(dead_code)]
#![allow(unused_variables)]

use std::collections::{hash_map::Entry, HashMap};

use leptos::{
    html::{Dialog, Output},
    leptos_dom::console_log,
    *,
};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{AnimationEvent, Event, KeyboardEvent};

const STORAGE_KEY: &str = "kalypsi";
const DEFAULT_WIDTH: usize = 6;
const DEFAULT_HEIGHT: usize = 6;

#[derive(PartialEq)]
struct Crossword {
    grid: Grid,
    answers: Answers,
}

struct Theme {
    black: String,
    background: String,
    primary: String,
    accent: String,
    text: String,
    surface: String,
    surface2: String,
}

impl Theme {
    fn to_css(&self) -> String {
        format!(
            "--background:{};--primary:{};--accent:{};--text:{};--black:{};--surface:{};--surface2:{}",
            self.background, self.primary, self.accent, self.text, self.black, self.surface, self.surface2
        )
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: "#1e1e2e".to_owned(),
            primary: "#f38ba8".to_owned(),
            accent: "#89b4fa".to_owned(),
            black: "#11111b".to_owned(),
            text: "#cdd6f4".to_owned(),
            surface: "#585b70".to_owned(),
            surface2: "#6c7086".to_owned(),
        }
    }
}

impl Crossword {
    fn new(cx: Scope) -> Self {
        let grid = Grid::new(cx);
        let mut answers = Answers::default();
        answers.refresh_map(&grid);

        Self { grid, answers }
    }

    fn toggle_cell(&mut self, cell: (usize, usize), cx: Scope) {
        if let Some(l) = self
            .grid
            .cells
            .chunks_exact_mut(self.grid.width)
            .nth(cell.0)
            .and_then(|c| c.get_mut(cell.1))
        {
            l.1 = match l.1 {
                Some(_) => None,
                None => Some(create_rw_signal(cx, ' ')),
            };
            self.answers.refresh_map(&self.grid);
        }
    }
}

#[derive(PartialEq)]
struct Grid {
    cells: Vec<(Uuid, Option<RwSignal<char>>)>,
    width: usize,
}

impl Grid {
    fn new(cx: Scope) -> Self {
        let cells = (0..DEFAULT_WIDTH * DEFAULT_HEIGHT)
            .map(|i| (Uuid::new_v4(), Some(create_rw_signal(cx, ' '))))
            .collect::<Vec<_>>();

        Self {
            cells,
            width: DEFAULT_WIDTH,
        }
    }

    fn get(&self, id: Uuid) -> Option<RwSignal<char>> {
        self.cells.iter().find(|s| s.0 == id).and_then(|s| s.1)
    }
}

impl<'a> BoolMatrix for &'a Grid {
    fn rows(&self) -> usize {
        self.cells.len() / self.width
    }

    fn cols(&self) -> usize {
        self.width
    }

    fn at(&self, cell: (usize, usize)) -> bool {
        self.cells
            .chunks_exact(self.width)
            .nth(cell.0)
            .and_then(|column| column.get(cell.1))
            .map_or(false, |(_, letter)| letter.is_some())
    }
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let grid = create_rw_signal(cx, Crossword::new(cx));

    let answer_map = create_memo(cx, move |_| grid.with(|g| g.answers.answer_map()));

    let cells = move || {
        grid.with(|c| {
            let val = c
                .grid
                .cells
                .iter()
                .copied()
                .enumerate()
                .map(|(i, cell)| ((i / c.grid.width, i % c.grid.width), cell))
                .collect::<Vec<_>>();
            val
        })
    };

    let letters = move || {
        grid.with(|c| {
            c.grid
                .cells
                .chunks_exact(c.grid.width)
                .map(|cc| cc.iter().map(|c| c.1.map(|s| s())).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        })
    };

    let selected_head: RwSignal<(Option<(usize, usize)>, bool)> =
        create_rw_signal(cx, (None, true));

    let selected_word = move || {
        let (current_word, is_across) = selected_head();
        if let Some(current_word) = current_word {
            grid.with(|g| {
                (
                    g.answers
                        .answer_keys
                        .get(&current_word)
                        .copied()
                        .and_then(|i| {
                            g.answers
                                .answers
                                .get(i)
                                .and_then(|h| {
                                    if is_across {
                                        h.across.as_ref()
                                    } else {
                                        h.down.as_ref()
                                    }
                                })
                                .map(|h| (i, h))
                        })
                        .map(|(i, h)| (i, h.answer.len())),
                    is_across,
                )
            })
        } else {
            (None, is_across)
        }
    };
    let caret_position: RwSignal<usize> = create_rw_signal(cx, 0);

    let caret_cell = move || {
        selected_head.with(|s| {
            s.0.map(|w| caret_position.with(|i| if s.1 { (w.0, w.1 + i) } else { (w.0 + i, w.1) }))
        })
    };

    let letter_at = move |cell: (usize, usize)| {
        grid.with(|c| {
            c.grid
                .cells
                .chunks_exact(c.grid.width)
                .nth(cell.0)
                .and_then(|column| column.get(cell.1))
                .and_then(|(_, letter)| *letter)
                .map(|letter| letter())
                .filter(|&l| l != ' ')
        })
    };

    let word_at = move |cell: Option<(usize, usize)>, is_across: bool| {
        cell.and_then(|c| {
            answer_map.with(|a| {
                if is_across {
                    a.across_run_keys.get(&c).copied()
                } else {
                    a.down_run_keys.get(&c).copied()
                }
            })
        })
    };

    let selected_head_with_len = move || {
        selected_head.with(move |selected| {
            grid.with(move |crossword| {
                selected.0.and_then(|v| {
                    if let Some(head) = &crossword.answers.head(v) {
                        if selected.1 {
                            head.across
                                .as_ref()
                                .map(|a| (v, a.answer.len(), selected.1))
                        } else {
                            head.down.as_ref().map(|a| (v, a.answer.len(), selected.1))
                        }
                    } else {
                        None
                    }
                })
            })
        })
    };

    let click_cell = move |coord: (usize, usize)| {
        let selected_cell = caret_cell();
        let (existing_selected, existing_is_across) = selected_head();
        let (new_selected, is_across) = answer_map.with(|crossword| {
            match (
                crossword.across_run_keys.get(&coord),
                crossword.down_run_keys.get(&coord),
                if selected_cell == Some(coord) {
                    !existing_is_across
                } else {
                    existing_is_across
                },
            ) {
                (Some(v), _, true) => (Some(*v), true),
                (_, Some(v), false) => (Some(*v), false),
                (None, Some(v), _) => (Some(*v), false),
                (Some(v), None, _) => (Some(*v), true),
                (None, None, _) => (None, existing_is_across),
            }
        });
        if (new_selected, is_across) != (existing_selected, existing_is_across) {
            selected_head.set((new_selected, is_across));
        }

        if let (Some(head), is_across) = selected_head() {
            caret_position.set(if is_across {
                coord.1 - head.1
            } else {
                coord.0 - head.0
            })
        };
    };

    let is_focused = move |coord: (usize, usize)| -> bool {
        selected_head_with_len().map_or(false, |(head, length, is_across)| {
            if is_across {
                coord.0 == head.0 && (head.1..head.1 + length).any(|v| v == coord.1)
            } else {
                coord.1 == head.1 && (head.0..head.0 + length).any(|v| v == coord.0)
            }
        })
    };

    let is_last_focused = move |coord: (usize, usize)| -> bool {
        selected_head_with_len().map_or(false, |(head, length, is_across)| {
            if is_across {
                coord.0 == head.0 && (head.1 + length - 1) == coord.1
            } else {
                coord.1 == head.1 && (head.0 + length - 1) == coord.0
            }
        })
    };

    let set_cell = move |selected_cell: (usize, usize), letter: char| {
        grid.with(|c| {
            if let Some((_, Some(cell))) = c
                .grid
                .cells
                .chunks_exact(c.grid.width)
                .nth(selected_cell.0)
                .and_then(|c| c.get(selected_cell.1))
            {
                cell.set(letter);
            }
        });
    };

    let next_word = move || {
        let (current_word, is_across) = selected_head();
        grid.with(|g| {
            if let Some(p) = g
                .answers
                .answers
                .iter()
                .skip(
                    current_word
                        .and_then(|c| g.answers.answer_keys.get(&c).copied())
                        .unwrap_or(0)
                        + 1,
                )
                .chain(g.answers.answers.iter())
                .find(|s| {
                    if is_across {
                        s.across.is_some()
                    } else {
                        s.down.is_some()
                    }
                })
            {
                selected_head.update(|s| s.0 = Some(p.head));
                caret_position.set(0);
            }
        });
    };

    let cell_answer_id = move |cell: (usize, usize)| {
        grid.with(|g| g.answers.answer_keys.get(&cell).copied().map(|i| i + 1))
    };

    let press_keydown = move |ev: Event| {
        // ev.prevent_default();
        let ev = ev.dyn_into::<KeyboardEvent>().unwrap();
        let selected_cell = match caret_cell() {
            Some(s) => s,
            _ => return,
        };

        match ev.key().as_str() {
            l if l.len() == 1 && l.is_ascii() => {
                let letter = l.chars().next().unwrap();
                set_cell(selected_cell, letter);
                if let Some((_, max_len, _)) = selected_head_with_len() {
                    caret_position.update(|c| *c = (*c + 1).min(max_len))
                }
            }
            "Backspace" => {
                if let Some((_, max_len, is_across)) = selected_head_with_len() {
                    let current_caret_position = caret_position();
                    match caret_cell().and_then(letter_at) {
                        Some(_) => set_cell(selected_cell, ' '),
                        None => {
                            if current_caret_position != 0 {
                                if is_across {
                                    set_cell((selected_cell.0, selected_cell.1 - 1), ' ');
                                } else {
                                    set_cell((selected_cell.0 - 1, selected_cell.1), ' ');
                                }
                            }
                            caret_position.update(|c| *c = c.saturating_sub(1))
                        }
                    }
                }
            }
            "Delete" => {
                grid.update(|g| g.toggle_cell(selected_cell, cx));
                // set_cell(selected_cell, None);
                selected_head.update(|existing| existing.0 = None);
            }
            "Tab" => {
                // next_word();
            }
            _ => {
                console_log(&ev.key());
            }
        }
    };

    window_event_listener("keydown", press_keydown);

    view! { cx,
        <div class="content">
            <div
                class="crossword"
                style=move || format!("--columns:{}", grid.with(| g | g.grid.width))
                across-entry-mode=move || selected_head.with(|s| s.1)
            >
                <For
                    each=cells
                    key=|a| a.1.0
                    view=move |cx, cell| {
                        view! { cx,
                            <Show
                                when=move || grid.with(|g| g.grid.get(cell.1.0).is_some())
                                fallback=|cx| {
                                    view! { cx, <div class="cell empty"></div> }
                                }
                            >
                                <Letter
                                    on:click=move |ev| click_cell(cell.0)
                                    focused=move || is_focused(cell.0)
                                    caret=move || caret_cell() == Some(cell.0)
                                    letter=move || cell.1.1.map_or(' ', |c| c())
                                    last_focused=move || is_last_focused(cell.0)
                                    answer_id=move || cell_answer_id(cell.0)
                                />
                            </Show>
                        }
                    }
                />
            </div>
            <Dialog/>
        </div>
    }
}

#[component]
pub fn Letter<S, F, L, G, A>(
    cx: Scope,
    letter: L,
    caret: S,
    focused: F,
    last_focused: G,
    answer_id: A,
) -> impl IntoView
where
    S: Fn() -> bool + 'static,
    F: Fn() -> bool + 'static + Copy,
    G: Fn() -> bool + 'static,
    L: Fn() -> char + 'static + Copy,
    A: Fn() -> Option<usize> + 'static,
{
    let animate = create_rw_signal(cx, false);

    // TODO: Is there a better way to do this?
    create_effect(cx, move |_| {
        let _ = letter();
        animate.set(true);
    });

    let remove_animation = move |ev: AnimationEvent| {
        animate.set(false);
    };

    view! { cx,
        <div class="cell" class:focused=focused class:caret=caret class:last-focused=last_focused>
            {move || {
                answer_id()
                    .map(|id| {
                        view! { cx, <span class="answer-id">{id}</span> }
                    })
            }}
            <span class="letter" class:animate=animate on:animationend=remove_animation>
                {letter}
            </span>
            <div class="decoration"></div>
        </div>
    }
}

#[component]
pub fn Dialog(cx: Scope) -> impl IntoView {
    let fav_dialog = create_node_ref::<Dialog>(cx);
    let output = create_node_ref::<Output>(cx);

    let theme = create_rw_signal(cx, Theme::default());

    create_effect(cx, move |_| {
        document()
            .body()
            .expect("Expected body to exist")
            .set_attribute("style", &theme.with(|t| t.to_css()))
            .expect("Expected to set attribute");
    });

    // let primary = move || theme.with(|t| t.primary.as_str());

    let show_dialog = move |ev| {
        fav_dialog
            .get()
            .expect("should be loaded")
            .show_modal()
            .expect("should show modal");
    };

    view! { cx,
        <dialog id="favDialog" _ref=fav_dialog>
            <form>
                <p>
                    <label>"Primary:" <input type="color"/></label>
                    <label>"Accent:" <input type="color"/></label>
                    <label>"Background:" <input type="color"/></label>
                </p>
                <div>
                    <button value="cancel" formmethod="dialog">
                        "Cancel"
                    </button>
                    <button id="confirmBtn" formmethod="dialog">
                        "Submit"
                    </button>
                </div>
            </form>
        </dialog>
        <p>
            <button id="showDialog" on:click=show_dialog>
                "Show the dialog"
            </button>
        </p>
        <output _ref=output></output>
    }
}

#[component]
pub fn BoardSettings(cx: Scope) -> impl IntoView {
    // let set_board = use_context::<WriteSignal<Board>>(cx).unwrap();
    //
    // let add_row = move |_| set_board.update(|b| b.add_row(cx));
    // let pop_row = move |_| set_board.update(|b| b.pop_row());
    // let add_column = move |_| set_board.update(|b| b.add_column(cx));
    // let pop_column = move |_| set_board.update(|b| b.pop_column());
    //
    // view! { cx,
    //     <div class="board-settings">
    //         <button on:click=add_row>"increase row"</button>
    //         <button on:click=pop_row>"descrease row"</button>
    //         <button on:click=add_column>"increase column"</button>
    //         <button on:click=pop_column>"decrease column"</button>
    //     </div>
    // }
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Slot {
    answer: String,
    clue: String,
}

#[derive(PartialEq, Debug, Clone, Default)]
pub struct Answers {
    answer_keys: HashMap<(usize, usize), usize>,
    answers: Vec<Head>,
}

impl Answers {
    fn head(&self, head: (usize, usize)) -> Option<&Head> {
        self.answer_keys
            .get(&head)
            .and_then(|&k| self.answers.get(k))
    }

    fn answer_map(&self) -> AnswerMap {
        let mut across_run_keys = HashMap::new();
        let mut down_run_keys = HashMap::new();

        for ((start, key), head) in self
            .answer_keys
            .iter()
            .filter_map(|k| self.answers.get(*k.1).map(|v| (k, v)))
        {
            if let Some(across) = &head.across {
                for i in 0..across.answer.len() {
                    across_run_keys.insert((start.0, start.1 + i), *start);
                }
            }

            if let Some(down) = &head.down {
                for i in 0..down.answer.len() {
                    down_run_keys.insert((start.0 + i, start.1), *start);
                }
            }
        }

        AnswerMap {
            across_run_keys,
            down_run_keys,
        }
    }

    fn refresh_map<M>(&mut self, m: M)
    where
        M: BoolMatrix,
    {
        let mut answer_keys = std::mem::take(&mut self.answer_keys);
        let mut answers = std::mem::take(&mut self.answers);

        for (x, y, length, is_across) in find_runs(m) {
            let coord = (x, y);

            let (new_head, answer_index) = match self.answer_keys.entry(coord) {
                Entry::Occupied(entry) => (&mut self.answers[*entry.get()], *entry.get()),
                Entry::Vacant(entry) => {
                    let answer_index = self.answers.len();
                    entry.insert(answer_index);
                    self.answers.push(Head {
                        head: coord,
                        down: None,
                        across: None,
                    });
                    (self.answers.last_mut().unwrap(), answer_index)
                }
            };

            if is_across {
                new_head.across = match answer_keys.get_mut(&coord).map(|v| &mut answers[*v]) {
                    Some(Head {
                        across: Some(existing),
                        ..
                    }) if existing.answer.len() == length => Some(std::mem::take(existing)),
                    _ => Some(Slot {
                        answer: ('a'..='z').take(length).collect(),
                        clue: "Enter a clue".to_owned(),
                    }),
                };
            } else {
                new_head.down = match answer_keys.get_mut(&coord).map(|v| &mut answers[*v]) {
                    Some(Head {
                        down: Some(existing),
                        ..
                    }) if existing.answer.len() == length => Some(std::mem::take(existing)),
                    _ => Some(Slot {
                        answer: ('a'..='z').take(length).collect(),
                        clue: "Enter a clue".to_owned(),
                    }),
                };
            }
        }
    }
}

#[derive(PartialEq, Debug)]
pub struct AnswerMap {
    across_run_keys: HashMap<(usize, usize), (usize, usize)>,
    down_run_keys: HashMap<(usize, usize), (usize, usize)>,
}

trait BoolMatrix {
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    fn at(&self, cell: (usize, usize)) -> bool;
}

fn find_runs<M>(m: M) -> Vec<(usize, usize, usize, bool)>
where
    M: BoolMatrix,
{
    let mut runs = Vec::new();
    let n_rows = m.rows();
    let n_cols = m.cols();

    // Find horizontal runs
    for i in 0..n_rows {
        let mut j = 0;
        while j < n_cols {
            if m.at((i, j)) {
                let head = j;
                let mut length = 1;
                j += 1;
                while j < n_cols && m.at((i, j)) {
                    length += 1;
                    j += 1;
                }
                if length > 1 {
                    runs.push((i, head, length, true));
                }
            } else {
                j += 1;
            }
        }
    }

    // Find vertical runs
    for j in 0..n_cols {
        let mut i = 0;
        while i < n_rows {
            if m.at((i, j)) {
                let head = i;
                let mut length = 1;
                i += 1;
                while i < n_rows && m.at((i, j)) {
                    length += 1;
                    i += 1;
                }
                if length > 1 {
                    runs.push((head, j, length, false));
                }
            } else {
                i += 1;
            }
        }
    }

    runs.sort_by_key(|(a, b, _, _)| a * n_cols + b);

    runs
}

#[derive(Clone, Default, Debug, PartialEq)]
struct Head {
    head: (usize, usize),
    down: Option<Slot>,
    across: Option<Slot>,
}

pub enum SlotDirection {
    Down,
    Across,
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn create_crossword() {
    //     let mut crossword = Crossword::new();
    //
    //     crossword.add_column_right();
    //     crossword.add_column_right();
    //     crossword.add_column_left();
    //
    //     crossword.add_row_bottom();
    //     crossword.add_row_bottom();
    //     crossword.add_row_bottom();
    //
    //     crossword.toggle_cell((0, 0));
    //     crossword.toggle_cell((0, 1));
    //     crossword.toggle_cell((1, 1));
    //     crossword.toggle_cell((2, 1));
    //
    //     dbg!(crossword.answer_keys);
    //     dbg!(crossword.answers);
    //     assert!(false)
    //     // assert_eq!(crossword.answers.len(), 2);
    // }
}
