use fltk::{
    app,
    button::Button,
    draw, enums,
    input::Input,
    prelude::*,
    text::{TextBuffer, TextDisplay},
    window::Window,
};
use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use crate::logic::{Board, CellState};

pub struct TicTacToeApp {}

#[derive(PartialEq)]
enum Outcomes {
    WIN,
    LOSS,
    DRAW,
    UNDEFINED,
}

impl TicTacToeApp {
    pub fn run() {
        let app = app::App::default();
        let font = app
            .load_font(Path::new("assets/FiraCode-Regular.ttf"))
            .unwrap();

        // Setup stage
        let wind = Rc::new(RefCell::new(Window::new(
            100,
            100,
            400,
            300,
            "Tic Tac Toe Setup",
        )));

        let mut board_size_input = Input::new(160, 50, 80, 30, "Board Size:");
        board_size_input.set_value("3");
        board_size_input.set_text_font(enums::Font::by_name(&font));
        board_size_input.set_label_font(enums::Font::by_name(&font));

        let mut win_condition_input = Input::new(160, 100, 80, 30, "Win Condition:");
        win_condition_input.set_value("3");
        win_condition_input.set_text_font(enums::Font::by_name(&font));
        win_condition_input.set_label_font(enums::Font::by_name(&font));

        let mut submit_button = Button::new(160, 150, 120, 60, "Start the game!");
        submit_button.set_label_font(enums::Font::by_name(&font));

        let board_size_clone = board_size_input.clone();
        let win_condition_clone = win_condition_input.clone();
        let wind_cloned = Rc::clone(&wind);

        let button_size = 45;

        submit_button.set_callback(move |_| {
            let board_size: usize = board_size_clone.value().parse().unwrap_or(3);
            let win_condition: usize = win_condition_clone.value().parse().unwrap_or(3);

            // Game stage
            let mut _board = Board::new(board_size, win_condition);
            let _board = Rc::new(RefCell::new(_board));
            let _board_cloned = _board.clone();

            let game_window_size = board_size as i32 * button_size;

            let game_wind = Rc::new(RefCell::new(Window::new(
                100,
                100,
                game_window_size,
                game_window_size,
                "Tic Tac Toe",
            )));
            game_wind.borrow_mut().make_resizable(true);
            let _game_wind_cloned = game_wind.clone();

            let play_again_button =
                Rc::new(RefCell::new(Button::new(220, 10, 100, 40, "Play Again")));
            play_again_button.borrow_mut().hide();
            let _play_again_button_cloned = play_again_button.clone();

            play_again_button.borrow_mut().set_callback(move |_| {
                _board_cloned.borrow_mut().reset();
                _play_again_button_cloned.borrow_mut().hide();
                _game_wind_cloned.borrow_mut().redraw();
            });

            // Draw lines between buttons
            {
                let game_wind_ref = game_wind.clone();
                game_wind.borrow_mut().draw(move |_: &mut Window| {
                    let wind = game_wind_ref.borrow_mut();
                    draw::set_draw_color(enums::Color::DarkGreen);
                    let sz_x = wind.width() / board_size as i32;
                    let sz_y = wind.height() / board_size as i32;

                    for i in 1..board_size {
                        draw::draw_line(i as i32 * sz_x, 0, i as i32 * sz_x, wind.height());
                        draw::draw_line(0, i as i32 * sz_y, wind.width(), i as i32 * sz_y);
                    }
                });

                // Handle resizing of game window
                let game_wind_ref_for_handle = game_wind.clone();
                game_wind.borrow_mut().handle(move |_, ev| {
                    if ev == enums::Event::Resize {
                        let mut wind = game_wind_ref_for_handle.borrow_mut();
                        wind.redraw();
                    }
                    true
                });
            }

            // Capture events on buttons
            for i in 0..board_size {
                for j in 0..board_size {
                    let mut cell = Button::new(
                        i as i32 * button_size,
                        j as i32 * button_size,
                        button_size,
                        button_size,
                        "",
                    );
                    cell.set_frame(enums::FrameType::BorderFrame);

                    let cell = Rc::new(RefCell::new(cell));
                    let cell_cloned = cell.clone();
                    let board = _board.clone();
                    let play_again_button_cloned = play_again_button.clone();
                    let font_cloned = font.clone();
                    let game_wind_cloned = game_wind.clone();

                    // Callback closure
                    cell.borrow_mut().set_callback(move |_| {
                        if board.borrow_mut().is_valid_move(i, j) {
                            let player = board.borrow_mut().get_current_player();
                            match player {
                                CellState::X => {
                                    cell_cloned.borrow_mut().set_label("X");
                                }
                                CellState::O => {
                                    cell_cloned.borrow_mut().set_label("O");
                                }
                                CellState::Empty => {}
                            };
                            board.borrow_mut().play_move(i, j);

                            // End conditions
                            let mut outcome = Outcomes::UNDEFINED;

                            // Win condition
                            if let Some(winner) = board.borrow_mut().is_winner() {
                                println!("Winner: {:?}", winner);
                                outcome = match winner {
                                    CellState::Empty => Outcomes::UNDEFINED,
                                    CellState::X => Outcomes::WIN,
                                    CellState::O => Outcomes::LOSS,
                                }
                            }

                            // Board full condition
                            if board.borrow_mut().is_board_full() {
                                println!("Board full");
                                outcome = Outcomes::DRAW;
                            }

                            if outcome != Outcomes::UNDEFINED {
                                game_wind_cloned.borrow_mut().hide();
                                // Create a new window
                                let mut result_wind = Window::new(200, 200, 400, 200, "Game Ended");

                                /*
                                let x_clip = game_wind_cloned.borrow().width();
                                let y_clip = game_wind_cloned.borrow().height();

                                draw::push_clip(0, 0, x_clip, y_clip);

                                let x_text = x_clip / 3;
                                let y_text = y_clip / 2;
                                let font_size = (y_text as f32 * 0.3) as i32;
                                println!(
                                    "width: {}, height: {}, font size: {}, outcome: {}",
                                    x_text, y_text, font_size, outcome
                                );
                                draw::set_draw_color(enums::Color::Red);
                                draw::draw_rect(x_text, y_text, 50, 50);
                                */
                                /*
                                draw::set_font(enums::Font::by_name(&font_cloned), font_size);
                                draw::draw_text(&outcome, x_text, y_text);
                                draw::pop_clip();
                                game_wind_cloned.borrow_mut().redraw();
                                */

                                /*
                                let label = &format!("Winner is {:?}", winner);
                                draw::set_font(enums::Font::by_name(&font_cloned), font_size);
                                draw::set_draw_color(enums::Color::Red);
                                draw::draw_text2(
                                    label,
                                    0,
                                    0,
                                    width,
                                    height,
                                    enums::Align::Center,
                                );
                                game_wind_cloned.borrow_mut().redraw();

                                play_again_button_cloned.borrow_mut().show();
                                */
                            }
                        }
                    });
                }
            }

            wind_cloned.borrow_mut().hide();
            game_wind.borrow_mut().end();
            game_wind.borrow_mut().show();
        });

        wind.borrow_mut().end();
        wind.borrow_mut().show();

        app.run().unwrap();
    }
}
