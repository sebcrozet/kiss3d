// use iced_wgpu::Renderer;
use iced_graphics::container;
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
    camera_reset_button: kiss3d_iced::widget::button::State,
    is_camera_reset_requested: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    BackgroundColorChanged(Color),
    TestEdited(String),
    CameraResetPressed,
    ClearCameraResetRequest,
}

impl Controls {
    pub fn new() -> Controls {
        Controls {
            background_color: Color::BLACK,
            sliders: Default::default(),
            test: String::new(),
            test_state: Default::default(),
            camera_reset_button: Default::default(),
            is_camera_reset_requested: false,
        }
    }

    pub fn background_color(&self) -> Color {
        self.background_color
    }

    pub fn is_camera_reset_requested(&self) -> bool {
        self.is_camera_reset_requested
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
            Message::CameraResetPressed => {
                self.is_camera_reset_requested = true;
            }
            Message::ClearCameraResetRequest => {
                self.is_camera_reset_requested = false;
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

        let inner = Column::new()
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
                    .push(
                        Row::new()
                            .align_items(Align::End)
                            .spacing(4)
                            .push(
                                kiss3d_iced::widget::Button::new(
                                    &mut self.camera_reset_button,
                                    Text::new("Reset Camera"),
                                )
                                .on_press(Message::CameraResetPressed),
                            )
                            .push(
                                kiss3d_iced::widget::TextInput::new(
                                    &mut self.test_state,
                                    "TextInput for test",
                                    &self.test,
                                    Message::TestEdited,
                                )
                                .padding(4)
                                .width(Length::Units(300)),
                            ),
                    ),
            );
        Row::new()
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::End)
            .push(
                container::Container::new(inner)
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .style(ContainerStyle),
            )
            .into()
    }
}

struct ContainerStyle;

impl container::StyleSheet for ContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(iced_graphics::Background::Color(Color::from_rgba(
                0.2, 0.2, 0.2, 0.4,
            ))),
            ..Default::default()
        }
    }
}
