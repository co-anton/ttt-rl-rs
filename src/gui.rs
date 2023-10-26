use fltk::{
    app, button::Button, draw, enums, frame::Frame, image::PngImage, input::Input, prelude::*,
    window::Window,
};
// use std::path::Path;
use std::cell::RefCell;
use std::rc::Rc;

use crate::{
    agent::{train, Action, QTable},
    logic::{Board, CellState},
};

pub struct TicTacToeApp {}

#[derive(PartialEq)]
enum Outcomes {
    Win,
    Loss,
    Draw,
    Undefined,
}

const WIN_IMAGE: &[u8] = include_bytes!("../assets/win.png");
const LOSS_IMAGE: &[u8] = include_bytes!("../assets/loss.png");
const DRAW_IMAGE: &[u8] = include_bytes!("../assets/draw.png");
const EMPTY_IMAGE: &[u8] = &[];

impl TicTacToeApp {
    pub fn run(train_agent: bool) {
        let app = app::App::default();
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

        let mut win_condition_input = Input::new(160, 100, 80, 30, "Win Condition:");
        win_condition_input.set_value("3");

        let mut submit_button = Button::new(160, 150, 120, 60, "Start the game!");

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

            // Train agent
            let mut agent: Option<Rc<RefCell<QTable>>> = None;
            if train_agent {
                agent = Some(Rc::new(RefCell::new(train(
                    1000,
                    100,
                    board_size,
                    win_condition,
                ))));
            }
            let game_window_size = board_size as i32 * button_size;

            let game_wind = Rc::new(RefCell::new(Window::new(
                100,
                100,
                game_window_size,
                game_window_size,
                "Tic Tac Toe",
            )));
            game_wind.borrow_mut().make_resizable(true);

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
                    let game_wind_cloned = game_wind.clone();
                    let agent_cloned = agent.clone(); // agent.as_ref().map(|agent_ref| agent_ref.clone());

                    // Callback closure
                    cell.borrow_mut().set_callback(move |_| {
                        println!("Attempting to play move [{}, {}]", i, j);
                        if board.borrow_mut().is_valid_move(i, j) {
                            let player = board.borrow_mut().get_current_player();
                            match player {
                                CellState::X => {
                                    cell_cloned.borrow_mut().set_label("X");
                                    match &agent_cloned {
                                        Some(agent_) => {
                                            let possible_actions = board
                                                .borrow_mut()
                                                .get_possible_actions()
                                                .iter()
                                                .map(|&(x_axis, y_axis)| Action { x_axis, y_axis })
                                                .collect::<Vec<Action>>();
                                            let action = agent_.borrow_mut().epsilon_greedy_search(
                                                &board.borrow_mut().get_grid(),
                                                &possible_actions,
                                            );
                                            println!("Agent play: {:?}", action);
                                        }
                                        None => {}
                                    }
                                }
                                CellState::O => {
                                    cell_cloned.borrow_mut().set_label("O");
                                }
                                CellState::Empty => {}
                            };
                            board.borrow_mut().play_move(i, j);
                            println!("Played move [{}, {}]", i, j);

                            // End conditions
                            let mut outcome = Outcomes::Undefined;

                            // Win condition
                            if let Some(winner) = board.borrow_mut().is_winner() {
                                println!("Winner: {:?}", winner);
                                outcome = match winner {
                                    CellState::Empty => Outcomes::Undefined,
                                    CellState::X => Outcomes::Win,
                                    CellState::O => Outcomes::Loss,
                                }
                            }

                            // Board full condition
                            if board.borrow_mut().is_board_full() && outcome == Outcomes::Undefined
                            {
                                println!("Board full");
                                outcome = Outcomes::Draw;
                            }

                            if outcome != Outcomes::Undefined {
                                game_wind_cloned.borrow_mut().hide();
                                // Create a new window
                                let result_wind = Rc::new(RefCell::new(Window::new(
                                    200,
                                    200,
                                    1024,
                                    1024,
                                    "Game Ended",
                                )));
                                let mut frame = Frame::new(0, 0, 1024, 1024, "");

                                let image_bytes = match outcome {
                                    Outcomes::Win => WIN_IMAGE,
                                    Outcomes::Draw => DRAW_IMAGE,
                                    Outcomes::Loss => LOSS_IMAGE,
                                    Outcomes::Undefined => EMPTY_IMAGE,
                                };

                                let image = match PngImage::from_data(image_bytes) {
                                    Ok(img) => img,
                                    Err(err) => {
                                        eprintln!("Cannot load image, error {}", err);
                                        return;
                                    }
                                };

                                frame.set_image(Some(image));

                                /*
                                let mut play_again_button = Button::new(
                                    1024 / 2,
                                    1024 / 4 - 200 / 2,
                                    200 / 2,
                                    200 / 2,
                                    "Play again?",
                                );

                                let result_wind_closed = result_wind.clone();
                                let board_cloned = board.clone();
                                play_again_button.set_callback(move |_| {
                                    result_wind_closed.borrow_mut().hide();
                                    board_cloned.borrow_mut().reset();
                                    // TODO go back to initial screen
                                });
                                */
                                result_wind.borrow_mut().end();
                                result_wind.borrow_mut().show();
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
