#![allow(clippy::type_complexity)]
#![allow(dead_code)]
#![allow(unused_variables)]

use std::{collections::HashMap, fmt::Display};

use leptos::{html::Div, leptos_dom::console_log, *};
use uuid::Uuid;
use wasm_bindgen::JsCast;
use web_sys::{Element, Event, FocusEvent, KeyboardEvent, MouseEvent};

use crate::generate::GridGenerator;

pub mod generate;
// pub mod state;
// pub mod util;

const STORAGE_KEY: &str = "kalypsi";
const DEFAULT_WIDTH: usize = 15;
const DEFAULT_HEIGHT: usize = 15;

#[derive(PartialEq)]
struct Store {
    crossword: Crossword,
    selection: Selection,
}

enum Action {
    ClickCell(usize, usize),
}

impl Store {
    fn dispatch(&self) -> impl Fn(Action) {
        |action| console_log("test")
    }
}

#[derive(PartialEq, Clone, Copy)]
struct Crossword {
    grid: RwSignal<Grid>,
    answers: Memo<Answers>,
    answer_map: Memo<AnswerMap>,
}

struct Theme {
    rosewater: &'static str,
    lavender: &'static str,
    text: &'static str,
    subtext1: &'static str,
    subtext0: &'static str,
    overlay2: &'static str,
    overlay1: &'static str,
    overlay0: &'static str,
    surface2: &'static str,
    surface1: &'static str,
    surface0: &'static str,
    base: &'static str,
    crust: &'static str,
    mantle: &'static str,
}

impl Theme {
    fn to_css(&self) -> String {
        format!(
"--cursor:{};--text:{};--crust:{};--surface0:{};--surface1:{};--surface2:{};--overlay0:{};--overlay1:{};--overlay2:{};--subtext0:{};--subtext1:{};--base:{};--crust:{};--mantle:{};",
            self.lavender, self.text, self.crust, self.surface0, self.surface1, self.surface2, self.overlay0, self.overlay1, self.overlay2, self.subtext0, self.subtext1, self.base, self.crust, self.mantle
        )
    }
}

impl Theme {
    fn catpuccin_mocha() -> Self {
        Self {
            rosewater: "#f5e0dc",
            lavender: "#b4befe",
            text: "#cdd6f4",
            subtext1: "#bac2de",
            subtext0: "#a6adc8",
            overlay2: "#9399b2",
            overlay1: "#7f849c",
            overlay0: "#6c7086",
            surface2: "#585b70",
            surface1: "#45475a",
            surface0: "#313244",
            base: "#1e1e2e",
            mantle: "#181825",
            crust: "#11111b",
        }
    }

    fn catpuccin_latte() -> Self {
        Self {
            rosewater: "#dc8a78",
            lavender: "#7287fd",
            text: "#4c4f69",
            subtext1: "#5c5f77",
            subtext0: "#6c6f85",
            overlay2: "#7c7f93",
            overlay1: "#8c8fa1",
            overlay0: "#9ca0b0",
            surface2: "#acb0be",
            surface1: "#bcc0cc",
            surface0: "#ccd0da",
            base: "#eff1f5",
            mantle: "#e6e9ef",
            crust: "#dce0e8",
        }
    }

    fn catpuccin_frappe() -> Self {
        Self {
            rosewater: "#f2d5cf",
            lavender: "#babbf1",
            text: "#c6d0f5",
            subtext1: "#b5bfe2",
            subtext0: "#a5adce",
            overlay2: "#949cbb",
            overlay1: "#838ba7",
            overlay0: "#737994",
            surface2: "#626880",
            surface1: "#51576d",
            surface0: "#414559",
            base: "#303446",
            mantle: "#292c3c",
            crust: "#232634",
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::catpuccin_mocha()
    }
}

impl Crossword {
    fn new(cx: Scope) -> Self {
        console_log("creating crossword");
        let grid = create_rw_signal(cx, Grid::new(cx));
        let answers = create_memo(cx, move |previous_answers| {
            grid.with(|g| Answers::new(previous_answers, g))
        });
        let answer_map = create_memo(cx, move |_| answers.with(|a| a.answer_map()));

        Self {
            grid,
            answers,
            answer_map,
        }
    }

    fn toggle_cell(&self, cx: Scope) -> impl Fn((usize, usize)) + Copy {
        let grid = self.grid;
        move |cell| {
            grid.update(|grid| {
                grid.toggle_cell(cell, cx);
            })
        }
    }

    fn display_cells(&self) -> impl Fn() -> Vec<((usize, usize), Option<Cell>)> {
        let grid = self.grid;

        move || {
            console_log("redisplaying");
            grid.with(|grid| {
                let val = grid
                    .cells
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(i, cell)| ((i % grid.width, i / grid.width), cell))
                    .collect::<Vec<_>>();
                val
            })
        }
    }

    fn cell_exists(&self) -> impl Fn((usize, usize)) -> bool + Copy {
        let grid = self.grid;

        move |coord: (usize, usize)| grid.with(|grid| grid.get(coord).and_then(|&c| c).is_some())
    }

    fn get_slot(&self) -> impl Fn((usize, usize), bool) -> Option<Slot> + Copy {
        let answer_map = self.answer_map;
        let answers = self.answers;
        move |cell, across| {
            answer_map.with(|answer_map| {
                answers.with(|answers| {
                    answer_map
                        .get(cell, across)
                        .and_then(|i| answers.answers.get(i))
                        .map(|s| Slot {
                            head: s.head,
                            len: s.word(across).as_ref().unwrap().answer.len(),
                            is_across: across,
                            caret_position: if across {
                                cell.0 - s.head.0
                            } else {
                                cell.1 - s.head.1
                            },
                        })
                })
            })
        }
    }

    fn set_cell(&self) -> impl Fn((usize, usize), char) + Copy {
        let grid = self.grid;

        move |cell, letter| grid.with(|g| g.set_cell(cell, letter))
    }

    fn corners_at(&self) -> impl Fn((usize, usize)) -> Corners + Copy {
        let grid = self.grid;
        move |cell| {
            grid.with(|grid| {
                let at = |c| grid.get(c).map_or(true, |c| c.is_none());
                let top = cell.1 == 0 || at((cell.0, cell.1 - 1));
                let left = cell.0 == 0 || at((cell.0 - 1, cell.1));
                let right = at((cell.0 + 1, cell.1));
                let bottom = at((cell.0, cell.1 + 1));
                let top_left = cell.1 == 0 || cell.0 == 0 || at((cell.0 - 1, cell.1 - 1));
                let top_right = cell.1 == 0 || at((cell.0 + 1, cell.1 - 1));
                let bottom_left = cell.0 == 0 || at((cell.0 - 1, cell.1 + 1));
                let bottom_right = at((cell.0 + 1, cell.1 + 1));

                Corners {
                    top_left: top && left,
                    top_right: top && right,
                    bottom_left: bottom && left,
                    bottom_right: bottom && right,
                }
            })
        }
    }

    fn answer_id_at(&self) -> impl Fn((usize, usize)) -> Option<usize> + Copy {
        let answers = self.answers;
        move |cell| answers.with(|answers| answers.answer_keys.get(&cell).copied().map(|id| id + 1))
    }

    fn style(&self) -> impl Fn() -> String + Copy {
        let grid = self.grid;

        move || {
            format!(
                "--columns:{};--rows:{}",
                grid.with(|g| g.width),
                grid.with(|g| g.cells.len() / g.width)
            )
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Cell {
    id: Uuid,
    letter: RwSignal<char>,
}

impl Cell {
    fn new(cx: Scope) -> Self {
        Self {
            id: Uuid::new_v4(),
            letter: create_rw_signal(cx, ' '),
        }
    }
}

#[derive(PartialEq, Debug)]
struct Grid {
    cells: Vec<Option<Cell>>,
    width: usize,
}

impl Grid {
    fn new(cx: Scope) -> Self {
        let cells = (0..DEFAULT_WIDTH * DEFAULT_HEIGHT)
            .map(|i| Some(Cell::new(cx)))
            .collect::<Vec<_>>();

        Self {
            cells,
            width: DEFAULT_WIDTH,
        }
    }

    fn black(&mut self, cell: (usize, usize), cx: Scope, black: bool) {
        if let Some(l) = self.get_mut(cell) {
            *l = match (*l, black) {
                (_, true) => None,
                (None, false) => Some(Cell::new(cx)),
                (Some(c), false) => Some(c),
            };
        }
    }

    fn toggle_cell(&mut self, cell: (usize, usize), cx: Scope) {
        if let Some(l) = self.get_mut(cell) {
            *l = match l {
                Some(_) => None,
                None => Some(Cell::new(cx)),
            };
        }
    }

    fn set_cell(&self, cell: (usize, usize), letter: char) {
        if let Some(cell) = self.char_at(cell) {
            cell.set(letter);
        }
    }

    fn get(&self, cell: (usize, usize)) -> Option<&Option<Cell>> {
        self.cells
            .chunks_exact(self.width)
            .nth(cell.1)
            .and_then(|column| column.get(cell.0))
    }

    fn get_mut(&mut self, cell: (usize, usize)) -> Option<&mut Option<Cell>> {
        self.cells
            .chunks_exact_mut(self.width)
            .nth(cell.1)
            .and_then(|row| row.get_mut(cell.0))
    }

    fn char_at(&self, cell: (usize, usize)) -> Option<RwSignal<char>> {
        self.get(cell).and_then(|&c| c).map(|c| c.letter)
    }
}

impl<'a> BoolMatrix for &'a Grid {
    fn rows(self) -> usize {
        self.cells.len() / self.width
    }

    fn cols(self) -> usize {
        self.width
    }

    fn at(self, cell: (usize, usize)) -> bool {
        self.get(cell).map_or(false, |c| c.is_some())
    }
}

impl<'a> TriBoolMatrix for &'a Grid {
    fn maybe_at(self, cell: (usize, usize)) -> Option<bool> {
        self.get(cell)
            .and_then(|c| c.map(|c| c.letter.get() == ' '))
    }
}

#[derive(PartialEq, Copy, Clone)]
struct Selection {
    active_slot: RwSignal<Option<Slot>>,
    default_is_across: Memo<bool>,
}

#[derive(PartialEq, Clone, Copy, Debug, Eq)]
pub struct Slot {
    head: (usize, usize),
    len: usize,
    is_across: bool,
    caret_position: usize,
}

impl Selection {
    fn new(cx: Scope) -> Self {
        let active_slot = create_rw_signal(cx, None);
        Self {
            active_slot,
            default_is_across: create_memo(cx, move |prev| {
                active_slot()
                    .map(|slot| slot.is_across)
                    .unwrap_or_else(|| prev.copied().unwrap_or(true))
            }),
        }
    }

    fn caret_cell(&self) -> impl Fn() -> Option<(usize, usize)> + Copy {
        let as_cc = self.active_slot_and_caret_cell();
        move || as_cc().map(|a| a.1)
    }

    fn active_slot_and_caret_cell(&self) -> impl Fn() -> Option<(Slot, (usize, usize))> + Copy {
        let active_word = self.active_slot;
        move || {
            active_word().map(|s| {
                if s.is_across {
                    (s, (s.head.0 + s.caret_position, s.head.1))
                } else {
                    (s, (s.head.0, s.head.1 + s.caret_position))
                }
            })
        }
    }

    fn caret_position(&self, cx: Scope) -> Signal<Option<usize>> {
        let active_slot = self.active_slot;
        Signal::derive(cx, move || {
            active_slot.with(|s| s.map(|s| s.caret_position))
        })
    }

    fn is_across(&self) -> impl Fn() -> bool + Copy {
        let active_slot = self.active_slot;

        move || active_slot().map_or(false, |s| s.is_across)
    }

    fn remove_selection(&self) -> impl Fn() + Copy {
        let active_slot = self.active_slot;

        move || {
            console_log("removing");
            active_slot.set(None);
        }
    }

    fn hide_caret(&self) -> impl Fn() -> bool + Copy {
        let active_slot = self.active_slot;
        move || active_slot().map_or(true, |a| a.caret_position >= a.len)
    }

    fn click_cell<S, C>(&self, get_slot: S, cell_exists: C) -> impl Fn((usize, usize)) + Copy
    where
        S: Fn((usize, usize), bool) -> Option<Slot> + Copy,
        C: Fn((usize, usize)) -> bool + Copy,
    {
        let existing_selection = self.active_slot_and_caret_cell();
        let active_slot = self.active_slot;
        let default_is_across = self.default_is_across;

        move |coord: (usize, usize)| {
            if !cell_exists(coord) {
                return;
            }

            let existing_selection = existing_selection();

            let use_across = existing_selection.map_or_else(
                default_is_across,
                |(existing_active_slot, caret_cell)| {
                    (caret_cell != coord) == existing_active_slot.is_across
                },
            );

            let new_slot = match (get_slot(coord, true), get_slot(coord, false)) {
                (Some(slot), _) if use_across => Some(slot),
                (_, Some(slot)) => Some(slot),
                (Some(slot), _) => Some(slot),
                _ => None,
            };

            if new_slot != existing_selection.map(|c| c.0) {
                active_slot.set(new_slot);
                default_is_across();
            }
        }
    }

    fn advance_caret(&self) -> impl Fn() + Copy {
        let active_slot = self.active_slot;
        move || {
            active_slot.update(|a| {
                if let Some(slot) = a.as_mut() {
                    slot.caret_position = slot.len.min(slot.caret_position + 1);
                }
            });
        }
    }

    fn retreat_caret(&self) -> impl Fn() + Copy {
        let active_slot = self.active_slot;
        move || {
            active_slot.update(|a| {
                if let Some(slot) = a.as_mut() {
                    slot.caret_position = slot.caret_position.saturating_sub(1);
                }
            });
        }
    }

    fn next_word<A>(&self, answers: A) -> impl Fn() + Copy
    where
        A: SignalWith<Answers> + Copy,
    {
        let active_slot = self.active_slot;
        move || {
            if let Some(slot) = active_slot() {
                answers.with(|answers| {
                    if let Some(new_slot) = answers.answer_keys.get(&slot.head).and_then(|&i| {
                        answers
                            .answers
                            .iter()
                            .skip(i + 1)
                            .chain(answers.answers.iter())
                            .find_map(|h| {
                                if slot.is_across {
                                    h.across.as_ref().map(|w| Slot {
                                        head: h.head,
                                        len: w.answer.len(),
                                        is_across: true,
                                        caret_position: 0,
                                    })
                                } else {
                                    h.down.as_ref().map(|w| Slot {
                                        head: h.head,
                                        len: w.answer.len(),
                                        is_across: false,
                                        caret_position: 0,
                                    })
                                }
                            })
                    }) {
                        active_slot.set(Some(new_slot));
                    }
                });
            }
        }
    }
}

fn log(is_across: impl Fn() -> bool + Copy + 'static) {
    if is_across() {
        console_log("animation frame");
        request_animation_frame(move || log(is_across));
    }
}
#[component]
pub fn App(cx: Scope) -> impl IntoView {
    // set_interval_with_handle(|| console_log("test"), duration);

    let crossword = Crossword::new(cx);
    provide_context(cx, crossword);
    let selection = Selection::new(cx);
    provide_context(cx, selection);

    let grid = crossword.grid;
    let answers = crossword.answers;

    let set_cell = crossword.set_cell();
    let toggle_cell = crossword.toggle_cell(cx);
    let next_word = selection.next_word(answers);

    let letter_at = move |cell: (usize, usize)| {
        grid.with(|grid| {
            grid.get(cell)
                .and_then(|&cell| cell.map(|c| c.letter.get()))
                .filter(|&l| l != ' ')
        })
    };

    let active_slot = selection.active_slot;
    let caret_position = selection.caret_position(cx);
    let caret_cell = selection.caret_cell();
    let remove_selection = selection.remove_selection();
    let remove_selection = move |ev: FocusEvent| {
        let target = ev.current_target().unwrap().dyn_into::<Element>().unwrap();
        if let Some(focus_target) = ev.related_target() {
            let focus_target = focus_target.dyn_into::<Element>().unwrap();
            if !target.contains(Some(&focus_target)) {
                remove_selection();
            }
        } else {
            remove_selection();
        }
    };
    let advance_caret = selection.advance_caret();
    let retreat_caret = selection.retreat_caret();
    let get_slot = crossword.get_slot();
    let cell_exists = crossword.cell_exists();
    let click_cell = selection.click_cell(get_slot, cell_exists);

    let press_keydown = move |ev: Event| {
        let ev = ev.dyn_into::<KeyboardEvent>().unwrap();

        match ev.key().as_str() {
            l if l.len() == 1 && l.is_ascii() => {
                ev.prevent_default();
                if let Some(selected_cell) = caret_cell() {
                    let letter = l.chars().next().unwrap();
                    set_cell(selected_cell, letter);
                    advance_caret();
                }
            }
            "Backspace" => {
                ev.prevent_default();
                if active_slot().is_some() {
                    if caret_cell().and_then(letter_at).is_none() {
                        retreat_caret();
                    }
                    if let Some(cell) = caret_cell() {
                        set_cell(cell, ' ');
                    }
                }
            }
            "Delete" => {
                ev.prevent_default();
                if let Some(selected_cell) = caret_cell() {
                    toggle_cell(selected_cell);
                    active_slot.update(|existing| *existing = None);
                }
            }
            "Tab" => {
                ev.prevent_default();
                next_word();
            }
            "ArrowRight" => {
                ev.prevent_default();
                if let Some(selected_cell) = caret_cell() {
                    let new_cell = (selected_cell.0 + 1, selected_cell.1);
                    click_cell(new_cell);
                }
            }
            "ArrowLeft" => {
                ev.prevent_default();
                if let Some(selected_cell) = caret_cell() {
                    if selected_cell.0 > 0 {
                        let new_cell = (selected_cell.0 - 1, selected_cell.1);
                        click_cell(new_cell);
                    }
                }
            }
            "ArrowUp" => {
                ev.prevent_default();
                if let Some(selected_cell) = caret_cell() {
                    if selected_cell.1 > 0 {
                        let new_cell = (selected_cell.0, selected_cell.1 - 1);
                        click_cell(new_cell);
                    }
                }
            }
            "ArrowDown" => {
                ev.prevent_default();
                if let Some(selected_cell) = caret_cell() {
                    let new_cell = (selected_cell.0, selected_cell.1 + 1);
                    click_cell(new_cell);
                }
            }
            "Control" => {
                ev.prevent_default();
                if let Some(selected_cell) = caret_cell() {
                    click_cell(selected_cell);
                }
            }
            _ => {
                console_log(&ev.key());
            }
        }
    };

    window_event_listener("keydown", press_keydown);

    let fill_blacks = move |_| {
        grid.update(move |grid| {
            let mut grid_generator = GridGenerator::new(&*grid);

            grid_generator.place_blacks(5.2..5.5, 40..73);

            let cells = grid_generator.cells();
            for (i, is_black) in cells.into_iter().enumerate() {
                let coord = (i % grid.width, i / grid.width);
                grid.black(coord, cx, is_black);
            }
        });
    };

    view! { cx,
        <div class="app">
            <div class="content">
                <Header/>
                <Crossword on:focusout=remove_selection/>
            </div>
            <button on:click=fill_blacks>"Fill blacks"</button>
            <Dialog/>
        </div>
    }
}

#[component]
pub fn Header(cx: Scope) -> impl IntoView {
    view! { cx,
        <div class="header">
            <h1>"Kalypsi"</h1>
        </div>
    }
}

#[component]
pub fn Crossword(cx: Scope) -> impl IntoView {
    let crossword = use_context::<Crossword>(cx).expect("Parent did not provide crossword");
    let selection = use_context::<Selection>(cx).expect("Parent did not provide selection");
    let style = crossword.style();
    let display_cells = crossword.display_cells();
    let cells = Signal::derive(cx, display_cells);
    let answer_id_at = crossword.answer_id_at();
    let corners_at = crossword.corners_at();
    let hide_caret = selection.hide_caret();
    let active_slot = selection.active_slot;
    let caret_cell = Signal::derive(cx, selection.caret_cell());
    let is_across = selection.is_across();

    create_effect(cx, move |_| log(is_across));

    let click_cell = selection.click_cell(crossword.get_slot(), crossword.cell_exists());

    view! { cx,
        <div class="crossword" style=style across-entry-mode=is_across>
            <Cells
                cells=cells
                answer_id_at=answer_id_at
                corners_at=corners_at
                click_cell=click_cell
                caret_cell=caret_cell
            />
            <Caret position=caret_cell hide=hide_caret/>
            <ActiveSlot position=active_slot/>
        </div>
    }
}

#[component]
pub fn Cells<A, O, C>(
    cx: Scope,
    cells: Signal<Vec<((usize, usize), Option<Cell>)>>,
    answer_id_at: A,
    corners_at: C,
    click_cell: O,
    caret_cell: Signal<Option<(usize, usize)>>,
) -> impl IntoView
where
    A: Fn((usize, usize)) -> Option<usize> + 'static + Copy,
    O: Fn((usize, usize)) + 'static + Copy,
    C: Fn((usize, usize)) -> Corners + 'static + Copy,
{
    view! { cx,
        <For
            each=cells
            key=|a| a.1.map_or_else(Uuid::new_v4, |i| i.id)
            view=move |cx, (position, cell)| {
                cell.map(|cell| {
                    let answer_id = Signal::derive(cx, move || answer_id_at(position));
                    let corners = Signal::derive(cx, move || corners_at(position));
                    let on_mouseover = move |ev: MouseEvent| {
                        if ev.buttons() == 1 && caret_cell().map_or(true, |c| c != position) {
                            click_cell(position);
                        }
                    };
                    view! { cx,
                        <Letter
                            on:click=move |ev| click_cell(position)
                            on:mouseover=on_mouseover
                            letter=cell.letter.into()
                            answer_id=answer_id
                            corners=corners
                            position=position
                        />
                    }
                })
            }
        />
    }
}

#[component]
pub fn Letter(
    cx: Scope,
    letter: Signal<char>,
    answer_id: Signal<Option<usize>>,
    corners: Signal<Corners>,
    position: (usize, usize),
) -> impl IntoView {
    let entered = create_rw_signal(cx, ());

    let is_entering = create_memo::<(Option<char>, bool)>(cx, move |prev_letter| {
        entered();
        let letter = letter();

        match prev_letter.and_then(|l| l.0) {
            None => (Some(letter), true),
            Some(prev_letter) if letter != prev_letter => (Some(letter), true),
            _ => (None, false),
        }
    });

    let is_entering = move || is_entering.get().1;

    let after_enter = move |_| {
        entered.set(());
    };

    let selection = use_context::<Selection>(cx).expect("Selection should be provided");
    let caret_cell = selection.caret_cell();
    let is_caret_cell = create_selector(cx, caret_cell);
    let node_ref = create_node_ref::<Div>(cx);
    create_effect(cx, move |_| {
        if is_caret_cell(Some(position)) {
            let node_ref = node_ref.get().expect("Should have node");
            let already_active = document()
                .active_element()
                .map_or(false, |e| node_ref.is_equal_node(Some(&e)));

            if !already_active {
                node_ref.focus().unwrap();
            }
        }
    });

    let style = move || format!("--x:{};--y:{}", position.0, position.1);

    let corner_bottom_left = move || corners().bottom_left;
    let corner_bottom_right = move || corners().bottom_right;
    let corner_top_left = move || corners().top_left;
    let corner_top_right = move || corners().top_right;

    view! { cx,
        <div
            tabindex=1
            class="cell"
            style=style
            _ref=node_ref
            class:corner-bottom-left=corner_bottom_left
            class:corner-bottom-right=corner_bottom_right
            class:corner-top-left=corner_top_left
            class:corner-top-right=corner_top_right
        >
            <span class="answer-id">{answer_id}</span>
            <span class="letter" class:enter=is_entering on:animationend=after_enter>
                {letter}
            </span>
        </div>
    }
}

#[derive(Clone, Copy)]
pub struct Corners {
    top_left: bool,
    top_right: bool,
    bottom_left: bool,
    bottom_right: bool,
}

#[component]
pub fn ActiveSlot<C>(cx: Scope, position: C) -> impl IntoView
where
    C: Fn() -> Option<Slot> + 'static + Copy,
{
    let entered = create_rw_signal(cx, ());
    let is_entering = create_memo::<(Option<Slot>, bool)>(cx, move |prev_position| {
        entered();
        let position = position();

        match (position, prev_position.and_then(|p| p.0)) {
            (Some(position), None) => (Some(position), true),
            (Some(position), Some(p)) if p.is_across != position.is_across => {
                (Some(position), true)
            }
            (position, _) => (position, false),
        }
    });

    let after_enter = move |_| {
        entered.set(());
    };

    let is_entering = move || is_entering.get().1;

    let style = move || {
        if let Some(slot) = position() {
            let caret = if slot.is_across {
                (slot.head.0 + slot.caret_position, slot.head.1)
            } else {
                (slot.head.0, slot.head.1 + slot.caret_position)
            };

            format!(
                "--x:{};--y:{};--caret-x:{};--caret-y:{};--length:{}",
                slot.head.0, slot.head.1, caret.0, caret.1, slot.len
            )
        } else {
            "".to_owned()
        }
    };

    let has_position = move || position().is_some();
    let across = move || position().map_or(false, |v| v.is_across);

    view! { cx,
        <Show when=has_position fallback=|_| ()>
            <div
                style=style
                class="slot"
                class:across=across
                class:enter=is_entering
                on:animationend=after_enter
            ></div>
        </Show>
    }
}

#[component]
pub fn Caret<C, H>(cx: Scope, position: C, hide: H) -> impl IntoView
where
    C: Fn() -> Option<(usize, usize)> + 'static + Copy,
    H: Fn() -> bool + 'static + Copy,
{
    let style = move || {
        if let Some(position) = position() {
            format!("--x:{};--y:{};", position.0, position.1)
        } else {
            "".to_owned()
        }
    };
    let has_position = move || position().is_some();

    view! { cx,
        <Show
            when=has_position
            fallback=move |cx| {
                view! { cx,  }
            }
        >
            <div style=style class="caret" class:hide=hide></div>
        </Show>
    }
}

#[component]
pub fn Dialog(cx: Scope) -> impl IntoView {
    let theme = create_rw_signal(cx, Theme::default());

    let animate = create_rw_signal(cx, false);

    let show_dialog = create_rw_signal(cx, false);
    create_effect(cx, move |_| {
        document()
            .body()
            .expect("Expected body to exist")
            .set_attribute("style", &theme.with(|t| t.to_css()))
            .expect("Expected to set attribute");
    });

    // let primary = move || theme.with(|t| t.primary.as_str());

    // let show_dialog = move |ev| {
    //     fav_dialog
    //         .get()
    //         .expect("should be loaded")
    //         .show_modal()
    //         .expect("should show modal");
    //     // animate.set(true);
    // };
    //

    view! { cx,
        <Show
            when=show_dialog
            fallback=move |cx| {
                view! { cx,  }
            }
        >
            <div class="dialog">
                <div class="dialog-content">
                    <p>
                        <label>"Primary:" <input type="color"/></label>
                        <label>"Accent:" <input type="color"/></label>
                        <label>"Background:" <input type="color"/></label>
                    </p>
                    <div>
                        <button value="cancel" on:click=move |_| show_dialog.set(false)>
                            "Cancel"
                        </button>
                        <button id="confirmBtn">"Submit"</button>
                    </div>
                </div>
            </div>
        </Show>
        <p>
            <button id="showDialog" on:click=move |_| show_dialog.set(true)>
                "Show the dialog"
            </button>
        </p>
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
pub struct Word {
    answer: Vec<RwSignal<char>>,
    clue: String,
}

impl Display for Word {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{ answer: {} }}",
            self.answer.iter().map(|s| s.get()).collect::<String>()
        )
    }
}

#[derive(PartialEq, Debug, Default)]
pub struct Answers {
    answer_keys: HashMap<(usize, usize), usize>,
    answers: Vec<Head>,
}

impl Answers {
    fn answer_map(&self) -> AnswerMap {
        console_log("creating answer map");
        let mut across_run_keys = HashMap::new();
        let mut down_run_keys = HashMap::new();

        for (answer_id, head) in self.answers.iter().enumerate() {
            if let Some(across) = &head.across {
                for i in 0..across.answer.len() {
                    across_run_keys.insert((head.head.0 + i, head.head.1), answer_id);
                }
            }

            if let Some(down) = &head.down {
                for i in 0..down.answer.len() {
                    down_run_keys.insert((head.head.0, head.head.1 + i), answer_id);
                }
            }
        }

        AnswerMap {
            across_run_keys,
            down_run_keys,
        }
    }

    fn new(this: Option<&Self>, grid: &Grid) -> Self {
        console_log("creating answers");
        let mut answers = Vec::<Head>::with_capacity(this.map_or(0, |t| t.answers.len()));
        let runs = find_runs(grid);

        let avg_len = runs.iter().fold(0, |acc, v| acc + v.2) as f32 / runs.len() as f32;
        console_log(&format!("average word len: {}", avg_len));
        let word_count = runs.len();
        console_log(&format!("word count: {}", word_count));

        for (x, y, length, is_across) in runs {
            let coord = (x, y);

            let new_head = match answers.iter_mut().find(|head| head.head == coord) {
                Some(new_head) => new_head,
                None => {
                    answers.push(Head {
                        head: coord,
                        down: None,
                        across: None,
                    });
                    answers.last_mut().unwrap()
                }
            };

            let word = Word {
                answer: (0..length)
                    .map(|i| {
                        let char_coord = if is_across {
                            (coord.0 + i, coord.1)
                        } else {
                            (coord.0, coord.1 + i)
                        };
                        grid.char_at(char_coord).unwrap()
                    })
                    .collect(),
                clue: this
                    .and_then(|t| {
                        t.get(coord)
                            .and_then(|a| a.word(is_across).as_ref())
                            .map(|w| w.clue.clone())
                    })
                    .unwrap_or("Enter a clue".to_owned()),
            };

            *new_head.word_mut(is_across) = Some(word);
        }

        Answers {
            answer_keys: answers
                .iter()
                .enumerate()
                .map(|(i, h)| (h.head, i))
                .collect(),
            answers,
        }
    }

    fn get(&self, coord: (usize, usize)) -> Option<&Head> {
        self.answer_keys
            .get(&coord)
            .and_then(|&i| self.answers.get(i))
    }
}

#[derive(PartialEq, Debug)]
pub struct AnswerMap {
    across_run_keys: HashMap<(usize, usize), usize>,
    down_run_keys: HashMap<(usize, usize), usize>,
}

impl AnswerMap {
    fn run_keys(&self, across: bool) -> &HashMap<(usize, usize), usize> {
        if across {
            &self.across_run_keys
        } else {
            &self.down_run_keys
        }
    }

    fn get(&self, cell: (usize, usize), across: bool) -> Option<usize> {
        self.run_keys(across).get(&cell).copied()
    }
}

pub trait TriBoolMatrix: BoolMatrix {
    fn maybe_at(self, cell: (usize, usize)) -> Option<bool>;
}

pub trait BoolMatrix: Copy {
    fn rows(self) -> usize;
    fn cols(self) -> usize;
    fn at(self, cell: (usize, usize)) -> bool;
}

fn find_runs<M>(m: M) -> Vec<(usize, usize, usize, bool)>
where
    M: BoolMatrix,
{
    let mut runs = Vec::new();
    let n_rows = m.rows();
    let n_cols = m.cols();

    // Find horizontal runs
    for y in 0..n_rows {
        let mut x = 0;
        while x < n_cols {
            if m.at((x, y)) {
                let x_start = x;
                let mut length = 1;
                x += 1;
                while x < n_cols && m.at((x, y)) {
                    length += 1;
                    x += 1;
                }
                if length > 1 {
                    runs.push((x_start, y, length, true));
                }
            } else {
                x += 1;
            }
        }
    }

    // Find vertical runs
    for x in 0..n_cols {
        let mut y = 0;
        while y < n_rows {
            if m.at((x, y)) {
                let y_start = y;
                let mut length = 1;
                y += 1;
                while y < n_rows && m.at((x, y)) {
                    length += 1;
                    y += 1;
                }
                if length > 1 {
                    runs.push((x, y_start, length, false));
                }
            } else {
                y += 1;
            }
        }
    }

    runs.sort_by_key(|(a, b, _, _)| a + n_cols * b);

    runs
}

#[derive(Clone, Default, Debug, PartialEq)]
struct Head {
    head: (usize, usize),
    down: Option<Word>,
    across: Option<Word>,
}

impl Head {
    fn word(&self, across: bool) -> &Option<Word> {
        if across {
            &self.across
        } else {
            &self.down
        }
    }

    fn word_mut(&mut self, across: bool) -> &mut Option<Word> {
        if across {
            &mut self.across
        } else {
            &mut self.down
        }
    }
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
