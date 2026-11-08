//! Máquina: one VM's settings (the basics, and more under "Mais opções"), the Mac it
//! presents, and deleting it. Changes go through `reims-ctl set`, deletion through
//! `reims-ctl delete`.

use crate::theme;
use crate::widgets;
use crate::{App, MaqField, Message, Screen};
use cuvee_core::i18n::Locale;
use cuvee_core::vm;
use iced::widget::{button, column, container, row, scrollable, slider, text_input, Column};
use iced::{Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let clock = app.clock();
    let t0 = app.entered_at;
    let m = &app.maquina;
    let running = app.maquina_running();
    let r0 = clock.rise(t0, 0);
    let r1 = clock.rise(t0, 1);
    let r2 = clock.rise(t0, 2);
    let r3 = clock.rise(t0, 3);

    let back = button(theme::label(
        app.t("exec-back"),
        14.0,
        theme::with_alpha(theme::MUTED, r0.opacity),
    ))
    .padding(0)
    .style(theme::ghost)
    .on_press(Message::Go(Screen::Painel));

    let mut header = column![
        back,
        theme::serif(
            m.name.clone(),
            72.0,
            theme::with_alpha(theme::INK, r0.opacity)
        ),
        theme::mono(
            m.dir.display().to_string(),
            13.0,
            theme::with_alpha(theme::MUTED, r0.opacity)
        ),
    ]
    .spacing(12);
    if running {
        header = header.push(theme::label(
            app.t("maq-running-note"),
            13.0,
            theme::with_alpha(theme::WARN, r0.opacity),
        ));
    }

    let settings: Element<'_, Message> = match (&m.saved, &m.edit, &m.load_error) {
        (Some(saved), Some(edit), _) => settings(app, saved, edit, r1.opacity),
        (_, _, Some(e)) => theme::label(
            app.locale
                .t_args("err-generic", &[("error", Locale::str(e))]),
            13.0,
            theme::with_alpha(theme::WARN, r1.opacity),
        )
        .into(),
        _ => theme::label("…", 13.0, theme::MUTED).into(),
    };

    let page = column![
        widgets::risen(r0, header),
        widgets::hairline(),
        widgets::risen(r1, settings),
        widgets::hairline(),
        widgets::risen(r2, identity(app, running, r2.opacity)),
        widgets::hairline(),
        widgets::risen(r3, danger(app, running, r3.opacity)),
        widgets::gap(24.0),
    ]
    .spacing(32)
    .max_width(760)
    .width(Length::Fill);

    scrollable(page)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(3).scroller_width(3),
        ))
        .height(Length::Fill)
        .into()
}

fn settings<'a>(app: &'a App, saved: &vm::Config, c: &vm::Config, a: f32) -> Element<'a, Message> {
    let m = &app.maquina;
    let mut col: Column<'a, Message> = column![
        slider_row(
            app.t("nova-cpus"),
            c.cpus.to_string(),
            1..=app.host_cpus.max(c.cpus).max(1),
            c.cpus,
            |n| Message::MaqEdit(MaqField::Cpus(n)),
            a
        ),
        slider_row(
            app.t("nova-memory"),
            format!("{} GB", c.memory_gib()),
            2..=app.host_mem_gib.max(c.memory_gib()).max(2),
            c.memory_gib(),
            |n| Message::MaqEdit(MaqField::MemoryGib(n)),
            a
        ),
    ]
    .spacing(22);

    // The rest, for whoever wants it.
    col = col.push(
        button(theme::label(
            app.t(if m.more { "maq-less" } else { "maq-more" }),
            14.0,
            theme::with_alpha(theme::INK, a),
        ))
        .padding([0, 2])
        .style(theme::ghost_bright)
        .on_press(Message::MaqMore),
    );
    if m.more {
        let resolution_ok = vm::valid_resolution(&c.resolution);
        col = col
            .push(slider_row(
                app.t("maq-displays"),
                c.displays.to_string(),
                1..=8,
                c.displays,
                |n| Message::MaqEdit(MaqField::Displays(n)),
                a,
            ))
            .push(field(
                app.t("maq-resolution"),
                text_input("1920x1080", &c.resolution)
                    .on_input(|s| Message::MaqEdit(MaqField::Resolution(s)))
                    .font(theme::MONO)
                    .size(15)
                    .padding(10)
                    .into(),
                if resolution_ok {
                    app.t("maq-resolution-hint")
                } else {
                    app.t("maq-resolution-bad")
                },
                if resolution_ok {
                    theme::SOFT
                } else {
                    theme::WARN
                },
                a,
            ))
            .push(switch(app, app.t("maq-hidpi"), c.hidpi, MaqField::Hidpi, a))
            .push(switch(
                app,
                app.t("maq-borderless"),
                c.borderless,
                MaqField::Borderless,
                a,
            ))
            .push(switch(app, app.t("maq-audio"), c.audio, MaqField::Audio, a))
            .push(switch(
                app,
                app.t("maq-network"),
                c.network,
                MaqField::Network,
                a,
            ))
            .push(field(
                app.t("maq-ssh"),
                text_input("0", &number(c.ssh_port as u64))
                    .on_input(|s| Message::MaqEdit(MaqField::SshPort(s)))
                    .font(theme::MONO)
                    .size(15)
                    .padding(10)
                    .width(Length::Fixed(160.0))
                    .into(),
                app.t("maq-ssh-hint"),
                theme::SOFT,
                a,
            ))
            .push(field(
                app.t("maq-stop-timeout"),
                text_input("90", &number(c.stop_timeout))
                    .on_input(|s| Message::MaqEdit(MaqField::StopTimeout(s)))
                    .font(theme::MONO)
                    .size(15)
                    .padding(10)
                    .width(Length::Fixed(160.0))
                    .into(),
                app.t("maq-stop-timeout-hint"),
                theme::SOFT,
                a,
            ));
        if c.graphics == "vmware-svga" {
            col = col.push(theme::label(
                app.t("maq-graphics-passthrough"),
                13.0,
                theme::with_alpha(theme::SOFT, a),
            ));
        }
    }

    // Save and undo appear once something differs from what reims-ctl has.
    if saved == c && !m.saving {
        return col.into();
    }
    let can_save = !m.saving && vm::valid_resolution(&c.resolution);
    let save = button(
        theme::label(
            if m.saving {
                "…".into()
            } else {
                app.t("maq-save")
            },
            15.0,
            theme::BG,
        )
        .font(theme::SANS_MEDIUM),
    )
    .padding([14, 26])
    .style(theme::cta)
    .on_press_maybe(can_save.then_some(Message::MaqSave));
    let mut actions = row![save].spacing(20).align_y(iced::Center);
    if !m.saving {
        actions = actions.push(
            button(theme::label(app.t("maq-undo"), 14.0, theme::MUTED))
                .padding([0, 2])
                .style(theme::ghost)
                .on_press(Message::MaqReset),
        );
    }
    col.push(actions).into()
}

/// The Mac the VM presents, and a way to give it another.
fn identity(app: &App, running: bool, a: f32) -> Element<'_, Message> {
    let m = &app.maquina;
    let busy = m.renewing || m.saving || m.deleting;
    let head = row![
        theme::label(
            app.t("nova-identity"),
            13.0,
            theme::with_alpha(theme::MUTED, a)
        ),
        iced::widget::space().width(Length::Fill),
        if !busy && !running && !m.confirm_renew && matches!(m.identity, Some(Ok(_))) {
            Element::from(
                button(theme::label(
                    app.t("nova-identity-again"),
                    13.0,
                    theme::with_alpha(theme::INK, a),
                ))
                .padding([0, 2])
                .style(theme::ghost_bright)
                .on_press(Message::MaqRenewAsk(true)),
            )
        } else {
            widgets::hgap(0.0).into()
        },
    ]
    .align_y(iced::Center);
    let body: Element<'_, Message> = match &m.identity {
        _ if m.renewing => theme::label(
            app.t("nova-identity-making"),
            13.0,
            theme::with_alpha(theme::SOFT, a),
        )
        .into(),
        Some(Ok(id)) => {
            let cell = |k: String, v: &str| {
                container(
                    column![
                        theme::label(k, 12.0, theme::with_alpha(theme::MUTED, a)),
                        theme::mono(v.to_string(), 13.0, theme::with_alpha(theme::INK, a)),
                    ]
                    .spacing(4),
                )
                .width(Length::Fill)
            };
            column![
                row![
                    cell(app.t("smbios-model"), &id.model),
                    cell(app.t("smbios-serial"), &id.serial),
                    cell("MLB".into(), &id.mlb),
                ]
                .spacing(24),
                row![cell("UUID".into(), &id.uuid), cell("ROM".into(), &id.rom)].spacing(24),
            ]
            .spacing(12)
            .into()
        }
        Some(Err(e)) => theme::label(
            app.locale
                .t_args("maq-identity-unreadable", &[("error", Locale::str(e))]),
            13.0,
            theme::with_alpha(theme::SOFT, a),
        )
        .into(),
        None => theme::label("…", 13.0, theme::MUTED).into(),
    };
    let mut col = column![head, body].spacing(12);
    if running {
        col = col.push(theme::label(
            app.t("maq-off-to-renew"),
            13.0,
            theme::with_alpha(theme::SOFT, a),
        ));
    }
    if m.confirm_renew {
        col = col.push(confirm(
            app.t("smbios-new-warning"),
            app.t("maq-renew-confirm"),
            Message::MaqRenew,
            Message::MaqRenewAsk(false),
            theme::WARN,
            app,
            a,
        ));
    }
    col.into()
}

/// Deleting the VM, behind a confirmation.
fn danger(app: &App, running: bool, a: f32) -> Element<'_, Message> {
    let m = &app.maquina;
    if m.confirm_delete {
        return confirm(
            app.locale
                .t_args("maq-delete-confirm", &[("vm", Locale::str(&m.name))]),
            if m.deleting {
                "…".into()
            } else {
                app.t("maq-delete-yes")
            },
            Message::MaqDelete,
            Message::MaqDeleteAsk(false),
            theme::BAD,
            app,
            a,
        );
    }
    let mut col = column![button(theme::label(
        app.t("maq-delete"),
        14.0,
        theme::with_alpha(if running { theme::DIM } else { theme::BAD }, a)
    ))
    .padding([0, 2])
    .style(theme::ghost)
    .on_press_maybe((!running && !m.saving && !m.renewing).then_some(Message::MaqDeleteAsk(true)))]
    .spacing(8);
    col = col.push(theme::label(
        app.t(if running {
            "maq-off-to-delete"
        } else {
            "maq-delete-hint"
        }),
        13.0,
        theme::with_alpha(theme::SOFT, a),
    ));
    col.into()
}

fn confirm<'a>(
    text: String,
    yes: String,
    on_yes: Message,
    on_no: Message,
    color: iced::Color,
    app: &App,
    a: f32,
) -> Element<'a, Message> {
    let busy = app.maquina.deleting || app.maquina.renewing;
    container(
        column![
            theme::label(text, 14.0, theme::with_alpha(theme::INK, a)),
            row![
                button(
                    theme::label(yes, 14.0, theme::with_alpha(color, a)).font(theme::SANS_MEDIUM)
                )
                .padding([0, 2])
                .style(theme::ghost)
                .on_press_maybe((!busy).then_some(on_yes)),
                button(theme::label(app.t("nova-cancel"), 14.0, theme::MUTED))
                    .padding([0, 2])
                    .style(theme::ghost)
                    .on_press_maybe((!busy).then_some(on_no)),
            ]
            .spacing(24),
        ]
        .spacing(16),
    )
    .padding([16, 20])
    .width(Length::Fill)
    .style(theme::code_block)
    .into()
}

fn slider_row<'a>(
    label: String,
    value_text: String,
    range: std::ops::RangeInclusive<u32>,
    value: u32,
    on_change: impl Fn(u32) -> Message + 'a,
    a: f32,
) -> Element<'a, Message> {
    column![
        row![
            theme::label(label, 13.0, theme::with_alpha(theme::MUTED, a)),
            iced::widget::space().width(Length::Fill),
            theme::serif(value_text, 30.0, theme::with_alpha(theme::INK, a)),
        ]
        .align_y(iced::alignment::Vertical::Bottom),
        slider(range, value, on_change).step(1u32),
    ]
    .spacing(10)
    .into()
}

fn field<'a>(
    label: String,
    input: Element<'a, Message>,
    hint: String,
    hint_color: iced::Color,
    a: f32,
) -> Element<'a, Message> {
    column![
        theme::label(label, 13.0, theme::with_alpha(theme::MUTED, a)),
        input,
        theme::label(hint, 12.0, theme::with_alpha(hint_color, a)),
    ]
    .spacing(8)
    .into()
}

/// A label with a two-way pill, as on Ajustes.
fn switch<'a>(
    app: &App,
    label: String,
    on: bool,
    field: impl Fn(bool) -> MaqField,
    a: f32,
) -> Element<'a, Message> {
    let option = |text: String, chosen: bool, value: bool| -> Element<'a, Message> {
        let t = theme::label(text, 13.0, if chosen { theme::BG } else { theme::MUTED })
            .font(theme::SANS_MEDIUM);
        if chosen {
            container(t)
                .padding([8, 16])
                .style(theme::tinted(theme::ACCENT))
                .into()
        } else {
            button(t)
                .padding([8, 16])
                .style(theme::ghost)
                .on_press(Message::MaqEdit(field(value)))
                .into()
        }
    };
    let pill = container(
        row![
            option(app.t("maq-on"), on, true),
            option(app.t("maq-off"), !on, false)
        ]
        .spacing(4),
    )
    .padding(4)
    .style(theme::tinted(theme::HOVER_BG));
    row![
        theme::label(label, 13.0, theme::with_alpha(theme::MUTED, a)).width(Length::Fill),
        pill,
    ]
    .align_y(iced::Center)
    .into()
}

/// A number for a text field: empty for 0, so the field can be cleared and typed over.
fn number(n: u64) -> String {
    if n == 0 {
        String::new()
    } else {
        n.to_string()
    }
}
