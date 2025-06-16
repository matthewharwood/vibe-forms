// main.rs
mod form_types;

use std::sync::Arc;
use axum::{
    extract::Form,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use axum::extract::State;
use form_macro::FormGen;
use maud::{html, Markup};
use serde::{Deserialize, Serialize};
use surrealdb::engine::remote::ws::{Client, Ws};
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;
use tokio::net::TcpListener;
use crate::form_types::{FormComponent, TextAreaField, TextField};

// Type alias for cleaner code
type WsClient = Client;

#[derive(Clone, Debug, Default, Serialize, Deserialize, FormGen)]
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

impl JobApplicationForm {
    pub async fn create(self, db: &Surreal<WsClient>) -> surrealdb::Result<Self> {
        let created: Option<Self> = db.create("job").content(self).await?;
        Ok(created.expect("create returned none"))
    }

    pub async fn get(db: &Surreal<WsClient>, id: &str) -> surrealdb::Result<Option<Self>> {
        db.select(("job", id)).await
    }

    pub async fn update(db: &Surreal<WsClient>, id: &str, data: &Self) -> surrealdb::Result<Option<Self>> {
        db.update(("job", id)).content(data.clone()).await
    }

    pub async fn delete(db: &Surreal<WsClient>, id: &str) -> surrealdb::Result<()> {
        db.delete(("job", id)).await.map(|_: Option<Self>| ())
    }
}

async fn show_form() -> Html<String> {
    let job = JobApplicationForm {
        name: TextField {
            label: "Full Name".to_string(),
            ..Default::default()
        },
        programming_story: TextAreaField {
            label: "5Ws got you into programming?".to_string(),
            ..Default::default()
        },
        ultimate_project: TextAreaField {
            label: "Ultimate project right now?".to_string(),
            ..Default::default()
        },
        proud_work: TextAreaField {
            label: "One piece of work you're most proud of".into(),
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

// This is your corrected form handler
async fn handle_form(State(app_state): State<Arc<AppState>>, form: Form<JobApplicationFormForm>) -> impl IntoResponse {
    let Form(form_data) = form;
    let job: JobApplicationForm = form_data.into();

    match job.create(&app_state.db).await {
        Ok(_rec) => Html(String::from("Success! Your application has been submitted.")),
        Err(e) => {
            eprintln!("Failed to insert: {:?}", e);
            Html(String::from("Error: Failed to submit application"))
        }
    }
}

fn render_form(p: &JobApplicationForm) -> Markup {
    html! {
        html {
            head {
                title { "Job Application Form" }
                style {
                    "
                    body { font-family: Arial, sans-serif; margin: 40px; }
                    form { max-width: 600px; }
                    label { display: block; margin-top: 20px; font-weight: bold; }
                    input, textarea { width: 100%; padding: 8px; margin-top: 5px; }
                    textarea { height: 100px; resize: vertical; }
                    button { margin-top: 20px; padding: 10px 20px; background: #007bff; color: white; border: none; cursor: pointer; }
                    button:hover { background: #0056b3; }
                    "
                }
            }
            body {
                h1 { "Job Application Form" }
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
                    button type="submit" { "Submit Application" }
                }
            }
        }
    }
}

struct AppState {
    db: Arc<Surreal<Client>>
}

#[tokio::main]
async fn main() {
    let db = match Surreal::new::<Ws>("127.0.0.1:8000").await {
        Ok(s) => {
            println!("Surreal instance created");
            s
        }
        Err(e) => {
            panic!("Surreal initialization error: {}", e);
        }
    };

    if let Err(e) = db
        .signin(Root {
            username: "root",
            password: "root",
        })
        .await
    {
        eprintln!("FATAL: Could not sign in to SurrealDB: {:?}", e);
        ::std::process::exit(1);
    };

    if let Err(e) = db.use_ns("test").use_db("test").await {
        eprintln!(
            "FATAL: Could not use namespace/database in SurrealDB: {:?}",
            e
        );
        ::std::process::exit(1);
    }

    let shared_db = Arc::new(db);
    let app_state = Arc::new(AppState {
        db: shared_db,
    });

    let app = Router::new()
        .route("/", get(show_form))
        .route("/submit", post(handle_form))
        .with_state(app_state);

    let addr: std::net::SocketAddr = "127.0.0.1:4000".parse().unwrap();
    println!("➡  Server running at http://{addr}");

    axum::serve(TcpListener::bind(addr).await.unwrap(), app.into_make_service())
        .await
        .unwrap();
}