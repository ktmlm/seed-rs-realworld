#[macro_use]
extern crate seed;
use seed::prelude::*;
use std::convert::TryInto;
use std::collections::VecDeque;

mod asset;
mod avatar;
mod username;
mod api;
mod viewer;
mod session;
mod login_form;
mod login_fetch;
mod register_form;
mod register_fetch;
mod page;
mod article;
mod route;

// Model

enum Model<'a> {
    Redirect(session::Session),
    NotFound(session::Session),
    Home(page::home::Model),
    Settings(page::settings::Model),
    Login(page::login::Model),
    Register(page::register::Model),
    Profile(page::profile::Model, username::Username<'a>),
    Article(page::article::Model),
    ArticleEditor(page::article_editor::Model, Option<article::slug::Slug<'a>>)
}

impl<'a> Default for Model<'a> {
    fn default() -> Self {
        Model::Redirect(session::Session::default())
    }
}

impl<'a> Model<'a> {
    pub fn take(&mut self) -> Model<'a> {
        std::mem::replace(self, Model::default())
    }
}

impl<'a> From<Model<'a>> for session::Session {
    fn from(model: Model<'a>) -> session::Session {
        match model {
            Model::Redirect(session) => session,
            Model::NotFound(session) => session,
            Model::Home(model) => model.into(),
            Model::Settings(model) => model.into(),
            Model::Login(model) => model.into(),
            Model::Register(model) => model.into(),
            Model::Profile(model, _) => model.into(),
            Model::Article(model) => model.into(),
            Model::ArticleEditor(model, _) => model.into(),
        }
    }
}

// Global msg handler

pub enum GMsg {
    RoutePushed(route::Route<'static>),
    SessionChanged(session::Session)
}

fn g_msg_handler<'a>(g_msg: GMsg, model: &mut Model<'a>, orders: &mut impl Orders<Msg<'static>, GMsg>) {
    if let GMsg::RoutePushed(ref route) = g_msg {
        orders.send_msg(Msg::ChangedRoute(Some(route.clone())));
    }

    match model {
        Model::NotFound(_) | Model::Redirect(_) => {
            if let GMsg::SessionChanged(session) = g_msg {
                orders.send_msg(Msg::GotSession(session));
            }
        },
        Model::Settings(model) => {
            page::settings::g_msg_handler(g_msg, model, &mut orders.proxy(Msg::GotSettingsMsg));
        },
        Model::Home(model) => {
            page::home::g_msg_handler(g_msg, model, &mut orders.proxy(Msg::GotHomeMsg));
        },
        Model::Login(model) => {
            page::login::g_msg_handler(g_msg, model, &mut orders.proxy(Msg::GotLoginMsg));
        },
        Model::Register(model) => {
            page::register::g_msg_handler(g_msg, model, &mut orders.proxy(Msg::GotRegisterMsg));
        },
        Model::Profile(model, _) => {
            page::profile::g_msg_handler(g_msg, model, &mut orders.proxy(Msg::GotProfileMsg));
        },
        Model::Article(model) => {
            page::article::g_msg_handler(g_msg, model, &mut orders.proxy(Msg::GotArticleMsg));
        },
        Model::ArticleEditor(model, _) => {
            page::article_editor::g_msg_handler(g_msg, model, &mut orders.proxy(Msg::GotArticleEditorMsg));
        },
    }
}

// Update

enum Msg<'a> {
    ChangedRoute(Option<route::Route<'a>>),
    GotSession(session::Session),
    GotHomeMsg(page::home::Msg),
    GotSettingsMsg(page::settings::Msg),
    GotLoginMsg(page::login::Msg),
    GotRegisterMsg(page::register::Msg),
    GotProfileMsg(page::profile::Msg),
    GotArticleMsg(page::article::Msg),
    GotArticleEditorMsg(page::article_editor::Msg),
}

fn update<'a>(msg: Msg<'a>, model: &mut Model<'a>, orders: &mut impl Orders<Msg<'static>, GMsg>) {
    match msg {
        Msg::ChangedRoute(route) => {
            change_model_by_route(route, model, orders);
        },
        Msg::GotSession(session) => {
            if let Model::Redirect(_) = model {
                *model = Model::Redirect(session);
                route::go_to(route::Route::Home, orders);
            }
        }
        Msg::GotHomeMsg(module_msg) => {
            if let Model::Home(module_model) = model {
                page::home::update(module_msg, module_model, &mut orders.proxy(Msg::GotHomeMsg));
            }
        },
        Msg::GotSettingsMsg(module_msg) => {
            if let Model::Settings(module_model) = model {
                page::settings::update(module_msg, module_model, &mut orders.proxy(Msg::GotSettingsMsg));
            }
        },
        Msg::GotLoginMsg(module_msg) => {
            if let Model::Login(module_model) = model {
                page::login::update(module_msg, module_model, &mut orders.proxy(Msg::GotLoginMsg));
            }
        },
        Msg::GotRegisterMsg(module_msg) => {
            if let Model::Register(module_model) = model {
                page::register::update(module_msg, module_model, &mut orders.proxy(Msg::GotRegisterMsg));
            }
        },
        Msg::GotProfileMsg(module_msg) => {
            if let Model::Profile(module_model, _) = model {
                page::profile::update(module_msg, module_model, &mut orders.proxy(Msg::GotProfileMsg));
            }
        },
        Msg::GotArticleMsg(module_msg) => {
            if let Model::Article(module_model) = model {
                page::article::update(module_msg, module_model, &mut orders.proxy(Msg::GotArticleMsg));
            }
        },
        Msg::GotArticleEditorMsg(module_msg) => {
            if let Model::ArticleEditor(module_model, _) = model {
                page::article_editor::update(module_msg, module_model, &mut orders.proxy(Msg::GotArticleEditorMsg));
            }
        },
    }
}

fn change_model_by_route<'a>(
    route: Option<route::Route<'a>>,
    model: &mut Model<'a>,
    orders:&mut impl Orders<Msg<'static>, GMsg>,
) {
    let mut session = || session::Session::from(model.take());
    match route {
        None => { *model = Model::NotFound(session()) },
        Some(route) => match route {
            route::Route::Root => {
                route::go_to(route::Route::Home, orders)
            },
            route::Route::Logout => {
                api::logout();
                orders.send_g_msg(GMsg::SessionChanged(None.into()));
                route::go_to(route::Route::Home, orders)
            },
            route::Route::NewArticle => {
                *model = Model::ArticleEditor(
                    page::article_editor::init_new(
                    session(), &mut orders.proxy(Msg::GotArticleEditorMsg)
                    ),
                    None
                );
            },
            route::Route::EditArticle(slug) => {
                *model = Model::ArticleEditor(
                    page::article_editor::init_edit(
                        session(), &slug, &mut orders.proxy(Msg::GotArticleEditorMsg)
                    ),
                    Some(slug)
                );
            },
            route::Route::Settings => {
                *model = Model::Settings(page::settings::init(
                    session(), &mut orders.proxy(Msg::GotSettingsMsg)
                ));
            },
            route::Route::Home => {
                *model = Model::Home(
                    page::home::init(session(), &mut orders.proxy(Msg::GotHomeMsg))
                );
            },
            route::Route::Login => {
                *model = Model::Login(
                    page::login::init(session(), &mut orders.proxy(Msg::GotLoginMsg))
                );
            },
            route::Route::Register => {
                *model = Model::Register(
                    page::register::init(session(),&mut orders.proxy(Msg::GotRegisterMsg))
                );
            },
            route::Route::Profile(username) => {
                *model = Model::Profile(
                    page::profile::init(
                        session(), &username, &mut orders.proxy(Msg::GotProfileMsg)
                    ),
                    username.into_owned()
                );
            },
            route::Route::Article(slug) => {
                *model = Model::Article(
                    page::article::init(session(), slug, &mut orders.proxy(Msg::GotArticleMsg))
                );
            },
        }
    };
}

// View

fn view<'a>(model: &Model) -> impl ElContainer<Msg<'static>> {
    match model {
        Model::Redirect(session) => {
            page::view(
                page::Page::Other,
                page::blank::view(),
                session.viewer(),
            )
        },
        Model::NotFound(session) => {
            page::view(
                page::Page::Other,
                page::not_found::view(),
                session.viewer(),
            )
        },
        Model::Settings(model) => {
            page::view(
                page::Page::Settings,
                page::settings::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotSettingsMsg)
        },
        Model::Home(model) => {
            page::view(
                page::Page::Home,
                page::home::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotHomeMsg)
        },
        Model::Login(model) => {
            page::view(
                page::Page::Login,
                page::login::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotLoginMsg)
        },
        Model::Register(model) => {
            page::view(
                page::Page::Register,
                page::register::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotRegisterMsg)
        },
        Model::Profile(model, username) => {
            page::view(
                page::Page::Profile(username),
                page::profile::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotProfileMsg)
        },
        Model::Article(model) => {
            page::view(
                page::Page::Other,
                page::article::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotArticleMsg)
        },
        Model::ArticleEditor(model, None) => {
            page::view(
                page::Page::NewArticle,
                page::article_editor::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotArticleEditorMsg)
        },
        Model::ArticleEditor(model, Some(_)) => {
            page::view(
                page::Page::Other,
                page::article_editor::view(model),
                model.session().viewer(),
            ).map_message(Msg::GotArticleEditorMsg)
        },
    }
}

// Init

fn init(url: Url, orders: &mut impl Orders<Msg<'static>, GMsg>) -> Model<'static> {
    orders.send_msg(Msg::ChangedRoute(url.try_into().ok()));
    Model::Redirect(api::load_viewer().into())
}

#[wasm_bindgen]
pub fn render() {
    seed::App::build(init, update, view)
        .routes(|url| {
            Msg::ChangedRoute(url.try_into().ok())
        })
        .g_msg_handler(g_msg_handler)
        .finish()
        .run();
}