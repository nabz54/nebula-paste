use super::*;
impl State {
    pub(super) fn board_view<'a>(
        &'a self,
        colors: nebula_paste::settings::TypeColors,
    ) -> Element<'a, Message> {
        let mut body=widget::column([]).spacing(8)
            .push(widget::text(tr!("Tableau · colonnes de cette collection","Board · columns in this collection")).size(18))
            .push(widget::text(tr!("Déplace une carte avec ← / →. Sans colonne conserve les éléments non affectés. Supprimer une colonne conserve ses éléments.","Move cards with ← / →. Unassigned holds items without a column. Deleting a column preserves its items.")).size(12))
            .push(widget::text_input(tr!("Nom de colonne","Column name"),&self.column_name).on_input(Message::ColumnName))
            .push(widget::flex_row(vec![widget::button::standard(if self.column_edit.is_some(){tr!("Renommer","Rename")}else{tr!("Ajouter une colonne","Add column")}).on_press(Message::ColumnSave).into(),widget::button::text(tr!("Nouvelle colonne","New column")).on_press(Message::ColumnEdit(None)).into()]).spacing(6));
        if let Some(id) = self.column_delete {
            body = body
                .push(widget::text(tr!(
                    "Supprimer cette colonne ? Ses éléments retourneront dans Sans colonne.",
                    "Delete this column? Its items return to Unassigned."
                )))
                .push(
                    widget::row([])
                        .push(
                            widget::button::destructive(tr!("Confirmer", "Confirm"))
                                .on_press(Message::ColumnDelete(id)),
                        )
                        .push(
                            widget::button::text(tr!("Annuler", "Cancel"))
                                .on_press(Message::ColumnAskDelete(None)),
                        ),
                );
        }
        let filtered = self.filtered();
        let page: Vec<_> = filtered
            .iter()
            .skip(self.page * 40)
            .take(40)
            .copied()
            .collect();
        let lanes: Vec<(Option<i64>, String)> =
            std::iter::once((None, tr!("Sans colonne", "Unassigned").into()))
                .chain(self.columns.iter().map(|c| (Some(c.id), c.name.clone())))
                .collect();
        let mut row = widget::row([]).spacing(10);
        for (index, (id, name)) in lanes.iter().enumerate() {
            let in_lane =
                |e: &&&Entry| self.assignments.get(&(e.kind, e.id.clone())).copied() == *id;
            let count = filtered.iter().filter(in_lane).count();
            let mut lane = widget::column([])
                .spacing(8)
                .push(widget::text(format!("{name} · {count}")).size(15));
            if let Some(id) = *id {
                lane = lane.push(
                    widget::flex_row(vec![
                        widget::button::text(tr!("Renommer", "Rename"))
                            .on_press(Message::ColumnEdit(Some(id)))
                            .into(),
                        widget::button::text("←")
                            .on_press_maybe((index > 1).then_some(Message::ColumnOrder(id, true)))
                            .into(),
                        widget::button::text("→")
                            .on_press_maybe(
                                (index + 1 < lanes.len())
                                    .then_some(Message::ColumnOrder(id, false)),
                            )
                            .into(),
                        widget::button::text(tr!("Supprimer…", "Delete…"))
                            .on_press(Message::ColumnAskDelete(Some(id)))
                            .into(),
                    ])
                    .spacing(2),
                );
            }
            for e in page.iter().filter(in_lane) {
                let mut actions = widget::row([]).spacing(4).push(
                    widget::button::text(if e.kind == 0 {
                        tr!("Copier", "Copy")
                    } else {
                        tr!("Ouvrir", "Open")
                    })
                    .on_press(Message::Open(e.kind, e.id.clone())),
                );
                if index > 0 {
                    actions = actions.push(widget::button::text("←").on_press(Message::BoardMove(
                        e.kind,
                        e.id.clone(),
                        lanes[index - 1].0,
                    )));
                }
                if index + 1 < lanes.len() {
                    actions = actions.push(widget::button::text("→").on_press(Message::BoardMove(
                        e.kind,
                        e.id.clone(),
                        lanes[index + 1].0,
                    )));
                }
                lane = lane.push(
                    widget::container(
                        widget::column([])
                            .spacing(6)
                            .push(crate::skin::type_badge(
                                e.label(),
                                e.icon(),
                                e.tint(),
                                colors,
                            ))
                            .push(
                                widget::text(e.title.chars().take(65).collect::<String>()).size(14),
                            )
                            .push(
                                widget::text(e.body.chars().take(120).collect::<String>()).size(12),
                            )
                            .push(actions),
                    )
                    .padding(10)
                    .width(Length::Fill)
                    .class(crate::skin::type_card(e.tint(), colors, false)),
                );
            }
            row = row.push(
                widget::container(lane)
                    .padding(10)
                    .width(260)
                    .class(cosmic::theme::Container::Card),
            );
        }
        body = body.push(widget::scrollable(row).direction(
            cosmic::iced::widget::scrollable::Direction::Horizontal(
                cosmic::iced::widget::scrollable::Scrollbar::default(),
            ),
        ));
        body.into()
    }
}
