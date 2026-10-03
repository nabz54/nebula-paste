use cosmic::{Element, widget};
use nebula_paste::{
    model::Kind,
    organize::{Match, Policy, Rule, RuleUndo},
    storage::Store,
    tr,
};
#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    Text(String),
    Kind,
    Destination(String),
    Save,
    Edit(i64),
    Remove(i64),
    ToggleRule(i64),
    Preview,
    Apply,
    Undo,
    Days(String),
    Exclude(String),
    PreviewCleanup,
    EnableCleanup,
    DisableCleanup,
}
#[derive(Default)]
pub struct State {
    pub open: bool,
    rules: Vec<Rule>,
    draft_id: i64,
    text: String,
    kind: Option<Kind>,
    destination: String,
    preview: Option<Vec<Match>>,
    undo: Option<RuleUndo>,
    days: String,
    excluded: Vec<String>,
    cleanup: Option<(Policy, Vec<(String, String)>)>,
    pub notice: String,
    policy: Policy,
    cleanup_titles: std::collections::HashMap<String, String>,
}
impl State {
    pub fn refresh(&mut self, s: &Store) -> Result<(), String> {
        self.rules = s.rules()?;
        self.policy = s.unused_policy()?;
        Ok(())
    }
    pub fn update(&mut self, m: Message, s: &Store) -> Result<(), String> {
        match m {
            Message::Toggle => {
                self.open = !self.open;
                self.days = self.policy.days.to_string();
                self.excluded = self.policy.excluded.clone();
                self.cleanup = None;
            }
            Message::Text(v) => {
                if v.len() <= 256 {
                    self.text = v;
                }
                self.preview = None;
            }
            Message::Kind => {
                self.kind = match self.kind {
                    None => Some(Kind::ALL[0]),
                    Some(k) => Kind::ALL
                        .iter()
                        .position(|v| *v == k)
                        .and_then(|i| Kind::ALL.get(i + 1).copied()),
                };
                self.preview = None;
            }
            Message::Destination(v) => {
                self.destination = v;
                self.preview = None;
            }
            Message::Save => {
                s.save_rule(&Rule {
                    id: self.draft_id,
                    kind: self.kind,
                    contains: self.text.clone(),
                    destination: self.destination.clone(),
                    enabled: true,
                })?;
                self.draft_id = 0;
                self.text.clear();
                self.preview = None;
            }
            Message::Edit(id) => {
                if let Some(r) = self.rules.iter().find(|r| r.id == id) {
                    self.draft_id = id;
                    self.kind = r.kind;
                    self.text = r.contains.clone();
                    self.destination = r.destination.clone();
                }
                self.preview = None;
            }
            Message::Remove(id) => {
                s.delete_rule(id)?;
                if self.draft_id == id {
                    self.draft_id = 0;
                    self.text.clear();
                }
                self.preview = None;
            }
            Message::ToggleRule(id) => {
                if let Some(r) = self.rules.iter().find(|r| r.id == id) {
                    let mut r = r.clone();
                    r.enabled = !r.enabled;
                    s.save_rule(&r)?;
                }
                self.preview = None;
            }
            Message::Preview => self.preview = Some(s.rule_preview()?),
            Message::Apply => {
                if let Some(p) = &self.preview {
                    self.undo = Some(s.apply_rules(p)?);
                    self.preview = None;
                    self.notice = tr!(
                        "Classement appliqué. Les conflits restent inchangés.",
                        "Filed. Conflicts remain unchanged."
                    )
                    .into();
                }
            }
            Message::Undo => {
                if let Some(u) = &self.undo {
                    s.undo_rules(u)?;
                    self.undo = None;
                    self.notice = tr!("Classement annulé.", "Filing undone.").into();
                }
            }
            Message::Days(v) => {
                if v.len() <= 4 {
                    self.days = v;
                }
                self.cleanup = None;
            }
            Message::Exclude(c) => {
                if self.excluded.contains(&c) {
                    self.excluded.retain(|x| x != &c);
                } else {
                    self.excluded.push(c);
                }
                self.cleanup = None;
            }
            Message::PreviewCleanup => {
                let p = Policy {
                    days: self.days.parse().map_err(|_| {
                        tr!("Délai requis : 1–3650 jours", "Delay required: 1–3650 days")
                    })?,
                    excluded: self.excluded.clone(),
                };
                if p.days == 0 {
                    return Err(tr!("Choisis un délai positif.", "Choose a positive delay.").into());
                }
                let rows = s.unused_preview(&p, nebula_paste::model::now())?;
                self.cleanup_titles = s.load()?.into_iter().map(|c| (c.id, c.title)).collect();
                self.cleanup = Some((p, rows));
            }
            Message::EnableCleanup => {
                if let Some((p, rows)) = &self.cleanup {
                    let now = nebula_paste::model::now();
                    let current = s.unused_preview(p, now)?;
                    if &current != rows {
                        self.cleanup = None;
                        return Err(tr!(
                            "La liste a changé : actualise l’aperçu.",
                            "List changed: refresh preview."
                        )
                        .into());
                    }
                    s.save_unused_policy(p)?;
                    s.expire_unused(now)?;
                    self.cleanup = None;
                    self.notice = tr!(
                        "Nettoyage automatique activé. Suppression définitive des copies expirées.",
                        "Automatic cleanup enabled. Expired clips are permanently deleted."
                    )
                    .into();
                }
            }
            Message::DisableCleanup => {
                s.save_unused_policy(&Policy::default())?;
                self.cleanup = None;
                self.days = "0".into();
            }
        }
        self.refresh(s)
    }
    pub fn view<'a>(&'a self, collections: &'a [String]) -> Element<'a, Message> {
        let mut b = widget::column([]).spacing(8).push(
            widget::button::text(tr!("Règles et nettoyage…", "Rules and cleanup…"))
                .on_press(Message::Toggle),
        );
        if !self.open {
            return b.into();
        }
        b=b.push(widget::text(tr!("Règles locales · nouvelles copies non classées, hors favoris","Local rules · new unfiled clips, excluding favorites")).size(15))
        .push(widget::text(tr!("Correspondance littérale sans distinction de casse/accents. Plusieurs destinations : aucun classement automatique. L’historique existant nécessite un aperçu et une confirmation.","Literal case/accent-insensitive matching. Multiple destinations: no automatic filing. Existing history requires preview and confirmation.")).size(12));
        for r in &self.rules {
            b = b.push(
                widget::flex_row(vec![
                    widget::text(format!(
                        "{} · {} → {}",
                        r.kind.map_or(tr!("Tous types", "All types"), Kind::label),
                        r.contains,
                        r.destination
                    ))
                    .into(),
                    widget::button::text(if r.enabled {
                        tr!("Désactiver", "Disable")
                    } else {
                        tr!("Activer", "Enable")
                    })
                    .on_press(Message::ToggleRule(r.id))
                    .into(),
                    widget::button::text(tr!("Modifier", "Edit"))
                        .on_press(Message::Edit(r.id))
                        .into(),
                    widget::button::text(tr!("Retirer la règle", "Remove rule"))
                        .on_press(Message::Remove(r.id))
                        .into(),
                ])
                .spacing(4),
            );
        }
        b = b
            .push(
                widget::text_input(tr!("Texte à rechercher", "Text to match"), &self.text)
                    .on_input(Message::Text),
            )
            .push(
                widget::button::text(
                    self.kind
                        .map_or(tr!("Tous types", "All types"), Kind::label),
                )
                .on_press(Message::Kind),
            );
        b = b
            .push(
                widget::flex_row(
                    collections
                        .iter()
                        .map(|c| {
                            widget::button::text(c)
                                .class(crate::skin::button(self.destination == *c, 8.0, false))
                                .on_press(Message::Destination(c.clone()))
                                .into()
                        })
                        .collect(),
                )
                .spacing(4),
            )
            .push(
                widget::button::standard(tr!("Enregistrer la règle", "Save rule")).on_press_maybe(
                    ((!self.text.trim().is_empty() || self.kind.is_some())
                        && collections.contains(&self.destination))
                    .then_some(Message::Save),
                ),
            )
            .push(
                widget::button::text(tr!("Prévisualiser sur l’historique", "Preview on history"))
                    .on_press(Message::Preview),
            );
        if let Some(p) = &self.preview {
            b = b.push(widget::text(format!(
                "{} · {}",
                p.len(),
                tr!("correspondances", "matches")
            )));
            for m in p {
                b = b.push(
                    widget::text(format!(
                        "{} · {} → {}{}",
                        m.title.chars().take(70).collect::<String>(),
                        m.before,
                        m.destinations.join(" / "),
                        if m.destinations.len() > 1 {
                            tr!(" · CONFLIT ignoré", " · CONFLICT skipped")
                        } else {
                            ""
                        }
                    ))
                    .size(12),
                );
            }
            b = b.push(
                widget::button::suggested(tr!(
                    "Appliquer les correspondances sans conflit",
                    "Apply unambiguous matches"
                ))
                .on_press_maybe(
                    p.iter()
                        .any(|m| m.destinations.len() == 1)
                        .then_some(Message::Apply),
                ),
            );
        }
        if self.undo.is_some() {
            b = b.push(
                widget::button::text(tr!("Annuler ce classement", "Undo this filing"))
                    .on_press(Message::Undo),
            );
        }
        b=b.push(widget::text(tr!("Expiration selon la dernière utilisation","Expiration by last use")).size(16))
        .push(widget::text(format!("{} : {}",tr!("Délai actif (jours, 0 = désactivé)","Active delay (days, 0 = disabled)"),self.policy.days)).size(12))
        .push(widget::text(tr!("Favoris, notes et modèles protégés. La rétention par âge et les limites de capacité restent indépendantes. Une copie réussie via l’historique remet le délai à zéro.","Favorites, notes and templates are protected. Age retention and capacity limits remain independent. A successful history copy resets the delay.")).size(12))
        .push(widget::text_input(tr!("Jours d’inactivité","Inactive days"),&self.days).on_input(Message::Days))
        .push(widget::text(tr!("Collections exclues","Excluded collections")))
        .push(widget::flex_row(collections.iter().map(|c|widget::button::text(c).class(crate::skin::button(self.excluded.contains(c),8.0,false)).on_press(Message::Exclude(c.clone())).into()).collect()).spacing(4))
        .push(widget::button::text(tr!("Prévisualiser le nettoyage","Preview cleanup")).on_press(Message::PreviewCleanup))
        .push(widget::button::text(tr!("Désactiver le nettoyage","Disable cleanup")).on_press(Message::DisableCleanup));
        if let Some((_, rows)) = &self.cleanup {
            b=b.push(widget::text(format!("{} · {}",rows.len(),tr!("copies seront supprimées définitivement. Sauvegarde conseillée avant activation.","clips will be permanently deleted. Back up before enabling."))));
            for (id, c) in rows {
                b = b.push(
                    widget::text(format!(
                        "{} · {}",
                        self.cleanup_titles
                            .get(id)
                            .unwrap_or(id)
                            .chars()
                            .take(70)
                            .collect::<String>(),
                        c
                    ))
                    .size(11),
                );
            }
            b = b.push(
                widget::button::destructive(tr!(
                    "Confirmer et activer le nettoyage automatique",
                    "Confirm and enable automatic cleanup"
                ))
                .on_press(Message::EnableCleanup),
            );
        }
        if !self.notice.is_empty() {
            b = b.push(widget::text(&self.notice));
        }
        b.into()
    }
}
