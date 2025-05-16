use iced::{Border, Length, Theme, widget};

use super::{Msg, PartCandidate};

#[derive(Debug, Clone)]
pub struct PendingBom {
    name: String,
    candidates: Vec<PartCandidate>,
}

#[derive(Debug, Clone)]
pub struct BomImporter {
    pending: Option<PendingBom>,
}

impl BomImporter {
    pub fn new() -> Self {
        Self { pending: None }
    }

    pub fn update(&mut self, msg: Msg) -> iced::Task<Msg> {
        match msg {
            Msg::LoadFile(path_buf) => todo!(),
            Msg::LoadSuccess(part_candidates) => todo!(),
            Msg::LoadFailed => todo!(),
            Msg::BomName(s) => {
                if let Some(pending) = &mut self.pending {
                    pending.name = s;
                }
            }
        }
        iced::Task::none()
    }

    pub fn view(&self) -> iced::Element<'_, Msg> {
        let mut bom_view = widget::Column::new();
        if let Some(pending) = &self.pending {
            bom_view = bom_view.extend(vec![
                widget::text("Name").into(),
                widget::text_input("", &pending.name)
                    .on_input(Msg::BomName)
                    .into(),
            ]);
        }

        widget::container(widget::column![widget::text("BOM Importer"), bom_view])
            .height(Length::Fill)
            .style(|theme: &Theme| {
                let palette = theme.extended_palette();
                widget::container::Style {
                    text_color: Some(palette.background.weak.text),
                    background: Some(palette.background.weak.color.into()),
                    border: Border::default().rounded(8.0),
                    ..Default::default()
                }
            })
            .width(Length::Fill)
            .padding(8.0)
            .into()
    }
}
