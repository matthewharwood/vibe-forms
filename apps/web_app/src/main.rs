mod form_types;

use axum::{
    extract::Form,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use form_macro::FormGen;
use maud::{html, Markup};
use serde::Serialize;
use serde_json::{json};
use tokio::net::TcpListener;
use crate::form_types::{FormComponent, TextAreaField, TextField};

#[derive(Clone, Debug, Default, Serialize, FormGen)]
pub struct JobApplicationForm {
    pub name:                TextField,
    pub programming_story:   TextAreaField,
    pub ultimate_project:    TextAreaField,
    pub proud_work:          TextAreaField,
    pub future_skills:       TextAreaField,
    pub oncall_stories:      TextAreaField,
    pub focus_strategies:    TextAreaField,
    pub support_systems:     TextAreaField,
    pub comfort_food:        TextAreaField,
    pub weekend:             TextAreaField,
    pub travel_wish:         TextAreaField,
}

async fn show_form() -> Html<String> {
    let job = JobApplicationForm {
        name: Default::default(),
        programming_story: TextAreaField {
            label: "5Ws got you into programming?".to_string(),
            ..Default::default()
        },
        ultimate_project: TextAreaField {
            label: "Ultimate project right now?".to_string(),
            ..Default::default()
        },
        proud_work: TextAreaField {
            label: "One piece of work you’re most proud of".into(),
            ..Default::default()
        },
        future_skills: TextAreaField {
            label: "Skills you want to master in 3-5 years".into(),
            ..Default::default()
        },
        oncall_stories: TextAreaField {
            label: "Any on-call horror stories?".into(),
            ..Default::default()
        },
        focus_strategies: TextAreaField {
            label: "Strategies to stay focused & present".into(),
            ..Default::default()
        },
        support_systems: TextAreaField {
            label: "Support systems you rely on & how we can help".into(),
            ..Default::default()
        },
        comfort_food: TextAreaField {
            label: "Favourite comfort food & why".into(),
            ..Default::default()
        },
        weekend: TextAreaField {
            label: "How do you spend your weekends?".into(),
            ..Default::default()
        },
        travel_wish: TextAreaField {
            label: "If you could travel anywhere …".into(),
            ..Default::default()
        },
    };
    Html(render_form(&job).into_string())
}

async fn handle_form(form: Form<JobApplicationFormForm>) -> impl IntoResponse {
    let Form(form_data) = form;
    let j: JobApplicationForm = form_data.into();

    (StatusCode::CREATED, json!(j).to_string())
}

fn render_form(p: &JobApplicationForm) -> Markup {
    html! {
        form method="post" action="/submit" {
            @match p {
                JobApplicationForm {
                    name,
                    programming_story,
                    ultimate_project,
                    proud_work,
                    future_skills,
                    oncall_stories,
                    focus_strategies,
                    support_systems,
                    comfort_food,
                    weekend,
                    travel_wish
                } => {
                    (name.render("name"))
                    (programming_story.render("programming_story"))
                    (ultimate_project.render("ultimate_project"))
                    (proud_work.render("proud_work"))
                    (future_skills.render("future_skills"))
                    (oncall_stories.render("oncall_stories"))
                    (focus_strategies.render("focus_strategies"))
                    (support_systems.render("support_systems"))
                    (comfort_food.render("comfort_food"))
                    (weekend.render("weekend"))
                    (travel_wish.render("travel_wish"))
                }
            }
            button { "Save" }
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(show_form))
        .route("/submit", post(handle_form));

    let addr: std::net::SocketAddr = "127.0.0.1:4000".parse().unwrap();
    println!("➡  Server running at http://{addr}");

    axum::serve(TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}