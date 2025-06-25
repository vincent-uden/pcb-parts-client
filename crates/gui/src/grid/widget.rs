use iced::{Border, Length, Padding, Shadow, Theme, alignment, widget};

use crate::settings;

use super::GridMessage;

const CELL_SIZE: f32 = 64.0;

#[derive(Debug)]
pub struct GridWidget {
    dimensions: settings::Grid,
    highlighted: Vec<(i64, i64)>,
    z: i64,
    selection_mode: bool,
    selected_cell: Option<(i64, i64)>,
}

impl GridWidget {
    pub fn new(dimensions: settings::Grid) -> Self {
        Self {
            dimensions,
            highlighted: vec![],
            z: 0,
            selection_mode: false,
            selected_cell: None,
        }
    }

    pub fn update(&mut self, message: GridMessage) -> iced::Task<GridMessage> {
        match message {
            GridMessage::HighlightParts(vec) => {
                self.highlighted = vec.iter().map(|p| (p.row, p.column)).collect();
                iced::Task::none()
            }
            GridMessage::LayerUp => todo!(),
            GridMessage::LayerDown => todo!(),
            GridMessage::CellClicked(row, column) => {
                if self.selection_mode {
                    self.selected_cell = Some((row, column));
                }
                iced::Task::none()
            }
            GridMessage::SetSelectionMode(enabled) => {
                self.selection_mode = enabled;
                if !enabled {
                    self.selected_cell = None;
                }
                iced::Task::none()
            }
        }
    }

    pub fn view(&self) -> iced::Element<'_, GridMessage> {
        let above = widget::column![];
        let below = widget::column![];

        let mut grid = widget::column![].spacing(8.0);
        for r in 0..self.dimensions.rows {
            let mut row: widget::Row<'_, GridMessage> = widget::row![].spacing(8.0);
            for c in 0..self.dimensions.columns {
                let is_highlighted = self.highlighted.contains(&(r, c));
                let is_selected = self.selected_cell == Some((r, c));
                
                let cell = widget::container("")
                    .width(CELL_SIZE)
                    .height(CELL_SIZE)
                    .style(move |theme: &Theme| {
                        let palette = theme.extended_palette();
                        if is_selected {
                            // Selected cell - green background
                            widget::container::Style {
                                border: Border::default().rounded(4.0),
                                background: Some(palette.success.base.color.into()),
                                shadow: Shadow {
                                    color: palette.success.base.color,
                                    offset: iced::Vector { x: 0.0, y: 0.0 },
                                    blur_radius: 8.0,
                                },
                                ..Default::default()
                            }
                        } else if is_highlighted {
                            // Highlighted cell - primary color
                            widget::container::Style {
                                border: Border::default().rounded(4.0),
                                background: Some(palette.primary.base.color.into()),
                                shadow: Shadow {
                                    color: palette.primary.base.color,
                                    offset: iced::Vector { x: 0.0, y: 0.0 },
                                    blur_radius: 8.0,
                                },
                                ..Default::default()
                            }
                        } else {
                            // Normal cell
                            widget::container::Style {
                                border: Border::default().rounded(4.0),
                                background: Some(palette.background.base.color.into()),
                                ..Default::default()
                            }
                        }
                    });

                if self.selection_mode {
                    // Make cell clickable in selection mode
                    row = row.push(
                        widget::mouse_area(cell)
                            .on_press(GridMessage::CellClicked(r, c))
                    );
                } else {
                    row = row.push(cell);
                }
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
        .padding(Padding::default().left(32.0).right(32.0))
        .width(Length::Shrink)
        .height(Length::Fill)
        .into()
    }
}
