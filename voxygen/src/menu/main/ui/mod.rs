mod connecting;
// Note: Keeping in case we re-add the disclaimer
//mod disclaimer;
mod credits;
mod login;
mod servers;
#[cfg(feature = "singleplayer")]
mod world_selector;

use crate::{
    GlobalState,
    credits::Credits,
    render::UiDrawer,
    ui::{
        self, Graphic,
        fonts::IcedFonts as Fonts,
        ice::{Element, IcedUi as Ui, load_font, style, widget},
        img_ids::ImageGraphic,
    },
    window,
};
use i18n::{LanguageMetadata, LocalizationHandle};
use iced::{Column, Container, HorizontalAlignment, Length, Row, Space, text_input};
use keyboard_keynames::key_layout::KeyLayout;
//ImageFrame, Tooltip,
use crate::settings::Settings;
use common::assets::{self, AssetExt};
use rand::{seq::SliceRandom, thread_rng};
use std::time::Duration;
use tracing::warn;

use super::DetailedInitializationStage;

// TODO: what is this? (showed up in rebase)
//const COL1: Color = Color::Rgba(0.07, 0.1, 0.1, 0.9);

pub const TEXT_COLOR: iced::Color = iced::Color::from_rgb(1.0, 1.0, 1.0);
pub const DISABLED_TEXT_COLOR: iced::Color = iced::Color::from_rgba(1.0, 1.0, 1.0, 0.2);

pub const FILL_FRAC_ONE: f32 = 0.67;
pub const FILL_FRAC_TWO: f32 = 0.53;

image_ids_ice! {
    struct Imgs {
        <ImageGraphic>
        v_logo: "voxygen.element.v_logo",
        bg: "voxygen.background.bg_main",
        banner_top: "voxygen.element.ui.generic.frames.banner_top",
        banner_gradient_bottom: "voxygen.element.ui.generic.frames.banner_gradient_bottom",
        button: "voxygen.element.ui.generic.buttons.button",
        button_hover: "voxygen.element.ui.generic.buttons.button_hover",
        button_press: "voxygen.element.ui.generic.buttons.button_press",
        input_bg: "voxygen.element.ui.generic.textbox",
        loading_art: "voxygen.element.ui.generic.frames.loading_screen.loading_bg",
        loading_art_l: "voxygen.element.ui.generic.frames.loading_screen.loading_bg_l",
        loading_art_r: "voxygen.element.ui.generic.frames.loading_screen.loading_bg_r",
        selection: "voxygen.element.ui.generic.frames.selection",
        selection_hover: "voxygen.element.ui.generic.frames.selection_hover",
        selection_press: "voxygen.element.ui.generic.frames.selection_press",

        #[cfg(feature = "singleplayer")]
        slider_range: "voxygen.element.ui.generic.slider.track",
        #[cfg(feature = "singleplayer")]
        slider_indicator: "voxygen.element.ui.generic.slider.indicator",

        unlock: "voxygen.element.ui.generic.buttons.unlock",
        unlock_hover: "voxygen.element.ui.generic.buttons.unlock_hover",
        unlock_press: "voxygen.element.ui.generic.buttons.unlock_press",
    }
}

// Randomly loaded background images
const BG_IMGS: [&str; 41] = [
    "voxygen.background.bg_1",
    "voxygen.background.bg_2",
    "voxygen.background.bg_3",
    "voxygen.background.bg_4",
    "voxygen.background.bg_5",
    "voxygen.background.bg_6",
    "voxygen.background.bg_7",
    "voxygen.background.bg_8",
    "voxygen.background.bg_9",
    "voxygen.background.bg_10",
    "voxygen.background.bg_11",
    "voxygen.background.bg_12",
    "voxygen.background.bg_13",
    "voxygen.background.bg_14",
    "voxygen.background.bg_15",
    "voxygen.background.bg_16",
    "voxygen.background.bg_17",
    "voxygen.background.bg_18",
    "voxygen.background.bg_19",
    "voxygen.background.bg_20",
    "voxygen.background.bg_21",
    "voxygen.background.bg_22",
    "voxygen.background.bg_23",
    "voxygen.background.bg_24",
    "voxygen.background.bg_25",
    "voxygen.background.bg_26",
    "voxygen.background.bg_27",
    "voxygen.background.bg_28",
    "voxygen.background.bg_29",
    "voxygen.background.bg_30",
    "voxygen.background.bg_31",
    "voxygen.background.bg_32",
    "voxygen.background.bg_33",
    "voxygen.background.bg_34",
    "voxygen.background.bg_35",
    "voxygen.background.bg_36",
    "voxygen.background.bg_37",
    "voxygen.background.bg_38",
    "voxygen.background.bg_39",
    "voxygen.background.bg_40",
    "voxygen.background.bg_41",
];

#[cfg(feature = "singleplayer")]
#[derive(Clone)]
pub enum WorldChange {
    Name(String),
    Seed(u32),
    DayLength(f64),
    SizeX(u32),
    SizeY(u32),
    Scale(f64),
    MapKind(common::resources::MapKind),
    ErosionQuality(f32),
    DefaultGenOps,
}

#[cfg(feature = "singleplayer")]
impl WorldChange {
    pub fn apply(self, world: &mut crate::singleplayer::SingleplayerWorld) {
        let mut def = Default::default();
        let gen_opts = world.gen_opts.as_mut().unwrap_or(&mut def);
        match self {
            WorldChange::Name(name) => world.name = name,
            WorldChange::Seed(seed) => world.seed = seed,
            WorldChange::DayLength(d) => world.day_length = d,
            WorldChange::SizeX(s) => gen_opts.x_lg = s,
            WorldChange::SizeY(s) => gen_opts.y_lg = s,
            WorldChange::Scale(scale) => gen_opts.scale = scale,
            WorldChange::MapKind(kind) => gen_opts.map_kind = kind,
            WorldChange::ErosionQuality(q) => gen_opts.erosion_quality = q,
            WorldChange::DefaultGenOps => world.gen_opts = Some(Default::default()),
        }
    }
}

#[cfg(feature = "singleplayer")]
#[derive(Clone)]
pub enum WorldsChange {
    SetActive(Option<usize>),
    Delete(usize),
    Regenerate(usize),
    AddNew,
    CurrentWorldChange(WorldChange),
}

pub enum Event {
    LoginAttempt {
        username: String,
        password: String,
        server_address: String,
    },
    CancelLoginAttempt,
    ChangeLanguage(LanguageMetadata),
    #[cfg(feature = "singleplayer")]
    StartSingleplayer,
    #[cfg(feature = "singleplayer")]
    InitSingleplayer,
    #[cfg(feature = "singleplayer")]
    SinglePlayerChange(WorldsChange),
    Quit,
    // Note: Keeping in case we re-add the disclaimer
    //DisclaimerAccepted,
    AuthServerTrust(String, bool),
    DeleteServer {
        server_index: usize,
    },
}

pub struct LoginInfo {
    pub username: String,
    pub password: String,
    pub server: String,
}

enum ConnectionState {
    InProgress,
    AuthTrustPrompt { auth_server: String, msg: String },
}

enum Screen {
    // Note: Keeping in case we re-add the disclaimer
    /*Disclaimer {
        screen: disclaimer::Screen,
    },*/
    Credits {
        screen: credits::Screen,
    },
    Login {
        screen: Box<login::Screen>, // boxed to avoid large variant
        // Error to display in a box
        error: Option<String>,
    },
    Servers {
        screen: servers::Screen,
    },
    Connecting {
        screen: connecting::Screen,
        connection_state: ConnectionState,
        init_stage: DetailedInitializationStage,
    },
    #[cfg(feature = "singleplayer")]
    WorldSelector {
        screen: world_selector::Screen,
    },
}

#[derive(PartialEq, Eq)]
enum Showing {
    Login,
    Languages,
}

impl Showing {
    fn toggle(&mut self, other: Showing) {
        if *self == other {
            *self = Showing::Login;
        } else {
            *self = other;
        }
    }
}

pub struct Controls {
    fonts: Fonts,
    imgs: Imgs,
    bg_img: widget::image::Handle,
    i18n: LocalizationHandle,
    // Voxygen version
    version: String,
    // Alpha disclaimer
    alpha: String,
    credits: Credits,

    // If a server address was provided via cli argument we hide the server list button and replace
    // the server field with a plain label (with a button to exit this mode and freely edit the
    // field).
    server_field_locked: bool,
    selected_server_index: Option<usize>,
    login_info: LoginInfo,

    show: Showing,
    selected_language_index: Option<usize>,

    time: f64,

    screen: Screen,
}

#[derive(Clone)]
enum Message {
    Quit,
    Back,
    ShowServers,
    ShowCredits,
    #[cfg(feature = "singleplayer")]
    Singleplayer,
    #[cfg(feature = "singleplayer")]
    SingleplayerPlay,
    #[cfg(feature = "singleplayer")]
    WorldChanged(WorldsChange),
    #[cfg(feature = "singleplayer")]
    WorldCancelConfirmation,
    #[cfg(feature = "singleplayer")]
    WorldConfirmation(world_selector::Confirmation),
    Multiplayer,
    UnlockServerField,
    LanguageChanged(usize),
    OpenLanguageMenu,
    Username(String),
    Password(String),
    Server(String),
    ServerChanged(usize),
    FocusPassword,
    CancelConnect,
    TrustPromptAdd,
    TrustPromptCancel,
    CloseError,
    DeleteServer,
    /* Note: Keeping in case we re-add the disclaimer
     *AcceptDisclaimer, */
}

impl Controls {
    fn new(
        fonts: Fonts,
        imgs: Imgs,
        bg_img: widget::image::Handle,
        i18n: LocalizationHandle,
        settings: &Settings,
        server: Option<String>,
    ) -> Self {
        let version = common::util::DISPLAY_VERSION_LONG.clone();
        let alpha = format!("Veloren {}", common::util::DISPLAY_VERSION.as_str());

        let credits = Credits::load_expect_cloned("credits");

        // Note: Keeping in case we re-add the disclaimer
        let screen = /* if settings.show_disclaimer {
            Screen::Disclaimer {
                screen: disclaimer::Screen::new(),
            }
        } else { */
            Screen::Login {
                screen: Box::default(),
                error: None,
            };
        //};

        let server_field_locked = server.is_some();
        let login_info = LoginInfo {
            username: settings.networking.username.clone(),
            password: String::new(),
            server: server.unwrap_or_else(|| settings.networking.default_server.clone()),
        };
        let selected_server_index = settings
            .networking
            .servers
            .iter()
            .position(|f| f == &login_info.server);

        let language_metadatas = i18n::list_localizations();
        let selected_language_index = language_metadatas
            .iter()
            .position(|f| f.language_identifier == settings.language.selected_language);

        Self {
            fonts,
            imgs,
            bg_img,
            i18n,
            version,
            alpha,
            credits,

            server_field_locked,
            selected_server_index,
            login_info,

            show: Showing::Login,
            selected_language_index,

            time: 0.0,

            screen,
        }
    }

    fn view(
        &mut self,
        settings: &Settings,
        key_layout: &Option<KeyLayout>,
        dt: f32,
        #[cfg(feature = "singleplayer")] worlds: &crate::singleplayer::SingleplayerWorlds,
    ) -> Element<Message> {
        self.time += dt as f64;

        // TODO: consider setting this as the default in the renderer
        let button_style = style::button::Style::new(self.imgs.button)
            .hover_image(self.imgs.button_hover)
            .press_image(self.imgs.button_press)
            .text_color(TEXT_COLOR)
            .disabled_text_color(DISABLED_TEXT_COLOR);

        let alpha = iced::Text::new(&self.alpha)
            .size(self.fonts.cyri.scale(12))
            .width(Length::Fill)
            .horizontal_alignment(HorizontalAlignment::Center);

        let top_text = Row::with_children(vec![
            Space::new(Length::Fill, Length::Shrink).into(),
            alpha.into(),
            if matches!(&self.screen, Screen::Login { .. }) {
                // Login screen shows the Velroen logo over the version
                Space::new(Length::Fill, Length::Shrink).into()
            } else {
                iced::Text::new(&self.version)
                    .size(self.fonts.cyri.scale(15))
                    .width(Length::Fill)
                    .horizontal_alignment(HorizontalAlignment::Right)
                    .into()
            },
        ])
        .padding(3)
        .width(Length::Fill);

        let bg_img = if matches!(&self.screen, Screen::Connecting { .. }) {
            self.bg_img
        } else {
            self.imgs.bg
        };

        let language_metadatas = i18n::list_localizations();

        // TODO: make any large text blocks scrollable so that if the area is to
        // small they can still be read
        let content = match &mut self.screen {
            // Note: Keeping in case we re-add the disclaimer
            //Screen::Disclaimer { screen } => screen.view(&self.fonts, &self.i18n, button_style),
            Screen::Credits { screen } => {
                screen.view(&self.fonts, &self.i18n.read(), &self.credits, button_style)
            },
            Screen::Login { screen, error } => screen.view(
                &self.fonts,
                &self.imgs,
                self.server_field_locked,
                &self.login_info,
                error.as_deref(),
                &self.i18n.read(),
                &self.show,
                self.selected_language_index,
                &language_metadatas,
                button_style,
                &self.version,
            ),
            Screen::Servers { screen } => screen.view(
                &self.fonts,
                &self.imgs,
                &settings.networking.servers,
                self.selected_server_index,
                &self.i18n.read(),
                button_style,
            ),
            Screen::Connecting {
                screen,
                connection_state,
                init_stage,
            } => screen.view(
                &self.fonts,
                &self.imgs,
                connection_state,
                init_stage,
                self.time,
                &self.i18n.read(),
                button_style,
                settings.interface.loading_tips,
                &settings.controls,
                key_layout,
            ),
            #[cfg(feature = "singleplayer")]
            Screen::WorldSelector { screen } => screen.view(
                &self.fonts,
                &self.imgs,
                worlds,
                &self.i18n.read(),
                button_style,
            ),
        };

        Container::new(
            Column::with_children(vec![top_text.into(), content])
                .spacing(3)
                .width(Length::Fill)
                .height(Length::Fill),
        )
        .style(style::container::Style::image(bg_img))
        .into()
    }

    fn update(
        &mut self,
        message: Message,
        events: &mut Vec<Event>,
        settings: &Settings,
        ui: &mut Ui,
    ) {
        let servers = &settings.networking.servers;
        let mut language_metadatas = i18n::list_localizations();

        match message {
            Message::Quit => events.push(Event::Quit),
            Message::Back => {
                self.screen = Screen::Login {
                    screen: Box::default(),
                    error: None,
                };
            },
            Message::ShowServers => {
                if matches!(&self.screen, Screen::Login { .. }) {
                    self.selected_server_index =
                        servers.iter().position(|f| f == &self.login_info.server);
                    self.screen = Screen::Servers {
                        screen: servers::Screen::new(),
                    };
                }
            },
            Message::ShowCredits => {
                self.screen = Screen::Credits {
                    screen: credits::Screen::new(),
                };
            },
            #[cfg(feature = "singleplayer")]
            Message::Singleplayer => {
                self.screen = Screen::WorldSelector {
                    screen: world_selector::Screen::default(),
                };
                events.push(Event::InitSingleplayer);
            },
            #[cfg(feature = "singleplayer")]
            Message::SingleplayerPlay => {
                self.screen = Screen::Connecting {
                    screen: connecting::Screen::new(ui),
                    connection_state: ConnectionState::InProgress,
                    init_stage: DetailedInitializationStage::Singleplayer,
                };
                events.push(Event::StartSingleplayer);
            },
            #[cfg(feature = "singleplayer")]
            Message::WorldChanged(change) => {
                match change {
                    WorldsChange::Delete(_) | WorldsChange::Regenerate(_) => {
                        if let Screen::WorldSelector {
                            screen: world_selector::Screen { confirmation, .. },
                        } = &mut self.screen
                        {
                            *confirmation = None;
                        }
                    },
                    _ => {},
                }
                events.push(Event::SinglePlayerChange(change))
            },
            #[cfg(feature = "singleplayer")]
            Message::WorldCancelConfirmation => {
                if let Screen::WorldSelector {
                    screen: world_selector::Screen { confirmation, .. },
                } = &mut self.screen
                {
                    *confirmation = None;
                }
            },
            #[cfg(feature = "singleplayer")]
            Message::WorldConfirmation(new_confirmation) => {
                if let Screen::WorldSelector {
                    screen: world_selector::Screen { confirmation, .. },
                } = &mut self.screen
                {
                    *confirmation = Some(new_confirmation);
                }
            },
            Message::Multiplayer => {
                self.screen = Screen::Connecting {
                    screen: connecting::Screen::new(ui),
                    connection_state: ConnectionState::InProgress,
                    init_stage: DetailedInitializationStage::StartingMultiplayer,
                };

                events.push(Event::LoginAttempt {
                    username: self.login_info.username.trim().to_string(),
                    password: self.login_info.password.clone(),
                    server_address: self.login_info.server.trim().to_string(),
                });
            },
            Message::UnlockServerField => self.server_field_locked = false,
            Message::Username(new_value) => self.login_info.username = new_value,
            Message::LanguageChanged(new_value) => {
                events.push(Event::ChangeLanguage(language_metadatas.remove(new_value)));
            },
            Message::OpenLanguageMenu => self.show.toggle(Showing::Languages),
            Message::Password(new_value) => self.login_info.password = new_value,
            Message::Server(new_value) => {
                self.login_info.server = new_value;
            },
            Message::ServerChanged(new_value) => {
                self.selected_server_index = Some(new_value);
                self.login_info.server.clone_from(&servers[new_value]);
            },
            Message::FocusPassword => {
                if let Screen::Login { screen, .. } = &mut self.screen {
                    screen.banner.password = text_input::State::focused();
                    screen.banner.username = text_input::State::new();
                }
            },
            Message::CancelConnect => {
                self.exit_connect_screen();
                events.push(Event::CancelLoginAttempt);
            },
            msg @ Message::TrustPromptAdd | msg @ Message::TrustPromptCancel => {
                if let Screen::Connecting {
                    connection_state, ..
                } = &mut self.screen
                {
                    if let ConnectionState::AuthTrustPrompt { auth_server, .. } = connection_state {
                        let auth_server = std::mem::take(auth_server);
                        let added = matches!(msg, Message::TrustPromptAdd);

                        *connection_state = ConnectionState::InProgress;
                        events.push(Event::AuthServerTrust(auth_server, added));
                    }
                }
            },
            Message::CloseError => {
                if let Screen::Login { error, .. } = &mut self.screen {
                    *error = None;
                }
            },
            Message::DeleteServer => {
                if let Some(server_index) = self.selected_server_index {
                    events.push(Event::DeleteServer { server_index });
                    self.selected_server_index = None;
                }
            },
            /* Note: Keeping in case we re-add the disclaimer */
            /*Message::AcceptDisclaimer => {
                if let Screen::Disclaimer { .. } = &self.screen {
                    events.push(Event::DisclaimerAccepted);
                    self.screen = Screen::Login {
                        screen: login::Screen::default(),
                        error: None,
                    };
                }
            },*/
        }
    }

    // Connection successful of failed
    fn exit_connect_screen(&mut self) {
        if matches!(&self.screen, Screen::Connecting { .. }) {
            self.screen = Screen::Login {
                screen: Box::default(),
                error: None,
            }
        }
    }

    fn auth_trust_prompt(&mut self, auth_server: String) {
        if let Screen::Connecting {
            connection_state, ..
        } = &mut self.screen
        {
            let msg = format!(
                "Warning: The server you are trying to connect to has provided this \
                 authentication server address:\n\n{}\n\nbut it is not in your list of trusted \
                 authentication servers.\n\nMake sure that you trust this site and owner to not \
                 try and bruteforce your password!",
                &auth_server
            );

            *connection_state = ConnectionState::AuthTrustPrompt { auth_server, msg };
        }
    }

    fn connection_error(&mut self, error: String) {
        if matches!(&self.screen, Screen::Connecting { .. })
            || matches!(&self.screen, Screen::Login { .. })
        {
            self.screen = Screen::Login {
                screen: Box::default(),
                error: Some(error),
            }
        } else {
            warn!("connection_error invoked on unhandled screen!");
        }
    }

    fn update_init_stage(&mut self, stage: DetailedInitializationStage) {
        if let Screen::Connecting { init_stage, .. } = &mut self.screen {
            *init_stage = stage
        }
    }

    fn tab(&mut self) {
        if let Screen::Login { screen, .. } = &mut self.screen {
            // TODO: add select all function in iced
            if screen.banner.username.is_focused() {
                screen.banner.username = text_input::State::new();
                screen.banner.password = text_input::State::focused();
                screen.banner.password.move_cursor_to_end();
            } else if screen.banner.password.is_focused() {
                screen.banner.password = text_input::State::new();
                // Skip focusing server field if it isn't editable!
                if self.server_field_locked {
                    screen.banner.username = text_input::State::focused();
                } else {
                    screen.banner.server = text_input::State::focused();
                }
                screen.banner.server.move_cursor_to_end();
            } else if screen.banner.server.is_focused() {
                screen.banner.server = text_input::State::new();
                screen.banner.username = text_input::State::focused();
                screen.banner.username.move_cursor_to_end();
            } else {
                screen.banner.username = text_input::State::focused();
                screen.banner.username.move_cursor_to_end();
            }
        }
    }
}

pub struct MainMenuUi {
    ui: Ui,
    // TODO: re add this
    // tip_no: u16,
    controls: Controls,
    bg_img_spec: &'static str,
}

impl MainMenuUi {
    pub fn new(global_state: &mut GlobalState) -> Self {
        // Load language
        let i18n = &global_state.i18n.read();
        // TODO: don't add default font twice
        let font = load_font(&i18n.fonts().get("cyri").unwrap().asset_key);

        let mut ui = Ui::new(
            &mut global_state.window,
            font,
            global_state.settings.interface.ui_scale,
        )
        .unwrap();

        let fonts = Fonts::load(i18n.fonts(), &mut ui).expect("Impossible to load fonts");

        let bg_img_spec = rand_bg_image_spec();

        let bg_img = assets::Image::load_expect(bg_img_spec).read().to_image();
        let controls = Controls::new(
            fonts,
            Imgs::load(&mut ui).expect("Failed to load images"),
            ui.add_graphic(Graphic::Image(bg_img, None)),
            global_state.i18n,
            &global_state.settings,
            global_state.args.server.clone(),
        );

        Self {
            ui,
            controls,
            bg_img_spec,
        }
    }

    pub fn bg_img_spec(&self) -> &'static str { self.bg_img_spec }

    pub fn update_language(&mut self, i18n: LocalizationHandle, settings: &Settings) {
        self.controls.i18n = i18n;
        let i18n = &i18n.read();
        let font = load_font(&i18n.fonts().get("cyri").unwrap().asset_key);
        self.ui.clear_fonts(font);
        self.controls.fonts =
            Fonts::load(i18n.fonts(), &mut self.ui).expect("Impossible to load fonts!");
        let language_metadatas = i18n::list_localizations();
        self.controls.selected_language_index = language_metadatas
            .iter()
            .position(|f| f.language_identifier == settings.language.selected_language);
    }

    pub fn auth_trust_prompt(&mut self, auth_server: String) {
        self.controls.auth_trust_prompt(auth_server);
    }

    pub fn show_info(&mut self, msg: String) { self.controls.connection_error(msg); }

    pub fn update_stage(&mut self, stage: DetailedInitializationStage) {
        tracing::trace!(?stage, "Updating stage");
        self.controls.update_init_stage(stage);
    }

    pub fn connected(&mut self) { self.controls.exit_connect_screen(); }

    pub fn cancel_connection(&mut self) { self.controls.exit_connect_screen(); }

    pub fn handle_event(&mut self, event: window::Event) -> bool {
        match event {
            // Pass events to ui.
            window::Event::IcedUi(event) => {
                self.handle_ui_event(event);
                true
            },
            window::Event::ScaleFactorChanged(s) => {
                self.ui.scale_factor_changed(s);
                false
            },
            _ => false,
        }
    }

    pub fn handle_ui_event(&mut self, event: ui::ice::Event) {
        // Tab for input fields
        use iced::keyboard;
        if matches!(
            &event,
            iced::Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Tab,
                ..
            })
        ) {
            self.controls.tab();
        }

        self.ui.handle_event(event);
    }

    pub fn set_scale_mode(&mut self, scale_mode: ui::ScaleMode) {
        self.ui.set_scaling_mode(scale_mode);
    }

    pub fn maintain(&mut self, global_state: &mut GlobalState, dt: Duration) -> Vec<Event> {
        let mut events = Vec::new();

        #[cfg(feature = "singleplayer")]
        let worlds_default = crate::singleplayer::SingleplayerWorlds::default();
        #[cfg(feature = "singleplayer")]
        let worlds = global_state
            .singleplayer
            .as_init()
            .unwrap_or(&worlds_default);

        let (messages, _) = self.ui.maintain(
            self.controls.view(
                &global_state.settings,
                &global_state.window.key_layout,
                dt.as_secs_f32(),
                #[cfg(feature = "singleplayer")]
                worlds,
            ),
            global_state.window.renderer_mut(),
            None,
            &mut global_state.clipboard,
        );

        messages.into_iter().for_each(|message| {
            self.controls
                .update(message, &mut events, &global_state.settings, &mut self.ui)
        });

        events
    }

    pub fn render<'a>(&'a self, drawer: &mut UiDrawer<'_, 'a>) { self.ui.render(drawer); }
}

pub fn rand_bg_image_spec() -> &'static str { BG_IMGS.choose(&mut thread_rng()).unwrap() }
