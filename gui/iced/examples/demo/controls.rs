// use iced_wgpu::Renderer;
use iced_native::{Align, Color, Command, Element, Length, Program};
use kiss3d_iced::{
    widget::{slider, Column, Row, Slider, Text},
    Renderer,
};

pub struct Controls {
    background_color: Color,
    sliders: [slider::State; 3],
    test: String,
    test_state: kiss3d_iced::widget::text_input::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(Color),
    TestEdited(String),
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            background_color: Color::BLACK,
            sliders: Default::default(),
            test: String::new(),
            test_state: Default::default(),
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }
}

impl Program for Controls {
    type Renderer = Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::BackgroundColorChanged(color) => {
                self.background_color = color;
            }
            Message::TestEdited(s) => {
                self.test = s;
            }
        }

        Command::none()
    }

    fn view(&mut self) -> Element<Message, Renderer> {
        let [r, g, b] = &mut self.sliders;
        let background_color = self.background_color;

        let sliders = Row::new()
            .width(Length::Units(500))
            .spacing(20)
            .push(
                Slider::new(r, 0.0..=1.0, background_color.r, move |r| {
                    Message::BackgroundColorChanged(Color {
                        r,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                Slider::new(g, 0.0..=1.0, background_color.g, move |g| {
                    Message::BackgroundColorChanged(Color {
                        g,
                        ..background_color
                    })
                })
                .step(0.01),
            )
            .push(
                Slider::new(b, 0.0..=1.0, background_color.b, move |b| {
                    Message::BackgroundColorChanged(Color {
                        b,
                        ..background_color
                    })
                })
                .step(0.01),
            );

        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::End)
            .push(
                Column::new()
                    .width(Length::Fill)
                    .align_items(Align::End)
                    .push(
                        Column::new()
                            .padding(10)
                            .spacing(10)
                            .push(Text::new("Background color").color(Color::WHITE))
                            .push(sliders)
                            .push(
                                Text::new(format!("{:?}", background_color))
                                    .size(14)
                                    .color(Color::WHITE),
                            )
                            .push(kiss3d_iced::widget::TextInput::new(
                                &mut self.test_state,
                                "TextInput for test",
                                &self.test,
                                Message::TestEdited,
                            )),
                    ),
            )
            .into()
    }
}
