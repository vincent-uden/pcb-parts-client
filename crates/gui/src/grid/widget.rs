use iced::{Border, Length, Theme, alignment, widget};

use crate::settings;

use super::GridMessage;

const CELL_SIZE: f32 = 48.0;

#[derive(Debug)]
pub struct GridWidget {
    dimensions: settings::Grid,
    z: i64,
}

impl GridWidget {
    pub fn new(dimensions: settings::Grid) -> Self {
        Self { dimensions, z: 0 }
    }

    pub fn update(message: GridMessage) -> iced::Task<GridMessage> {
        todo!()
    }

    pub fn view(&self) -> iced::Element<'_, GridMessage> {
        let above = widget::column![];
        let below = widget::column![];

        let mut grid = widget::column![].spacing(4.0);
        for _ in 0..self.dimensions.rows {
            let mut row: widget::Row<'_, GridMessage> = widget::row![].spacing(4.0);
            for _ in 0..self.dimensions.columns {
                row = row.push(
                    widget::container("")
                        .width(CELL_SIZE)
                        .height(CELL_SIZE)
                        .style(|theme: &Theme| {
                            let palette = theme.extended_palette();
                            widget::container::Style {
                                border: Border::default().rounded(4.0),
                                background: Some(palette.background.base.color.into()),
                                ..Default::default()
                            }
                        }),
                );
            }
            grid = grid.push(row);
        }
        let grid_con = widget::container(grid)
            .padding(16.0)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                widget::container::Style {
                    border: Border::default().rounded(8.0),
                    background: Some(palette.background.weak.color.into()),
                    ..Default::default()
                }
            });

        widget::column![
            widget::vertical_space().height(Length::Fill),
            above,
            grid_con,
            below,
            widget::vertical_space().height(Length::Fill)
        ]
        .align_x(alignment::Horizontal::Center)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
}
