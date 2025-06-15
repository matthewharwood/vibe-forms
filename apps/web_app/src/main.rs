use axum::{
    extract::Form,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use form_macro::FormGen;  // ← Only import FormGen, not form
use maud::{html, Markup};
use serde::Serialize;  // ← Removed unused Deserialize
use serde_json::json;
use tokio::net::TcpListener;

// ── domain models with tree-walking form generation ────
#[derive(Clone, Debug, Default, Serialize, FormGen)]
pub struct Field {
    #[mark]
    value: String,      // ← This gets extracted to forms as "{parent}_value"
    label: String,      // ← This stays in domain only  
    input_type: String, // ← This stays in domain only
}

#[derive(Clone, Debug, Default, Serialize, FormGen)]
pub struct Person {
    name: Field,        // ← Macro finds marked "value" inside → becomes "name_value: String"
    zip: Field,         // ← Macro finds marked "value" inside → becomes "zip_value: String"
    #[mark]
    age: u8,           // ← Marked primitive → becomes "age: u8"
}

// ── The macro auto-generates PersonForm: ───────────────
// pub struct PersonForm {
//     pub name_value: String,  // from name.value (marked in Field)
//     pub zip_value: String,   // from zip.value (marked in Field) 
//     pub age: u8,            // from age (marked in Person)
// }
// + From<PersonForm> for Person
// + From<Person> for PersonForm
pub enum TypeMap {
    TEXT
}
// ── show the HTML form ─────────────────────────────────
async fn show_form() -> Html<String> {
    let person = Person {
        name: Field {
            value: "".into(),
            label: "Name".into(),
            input_type: "text".into(),
        },
        zip: Field {
            value: "".into(),
            label: "ZIP".into(),
            input_type: "text".into(),
        },
        age: 0,
    };
    Html(render_form(&person).into_string())
}

// ── handle POST (beautifully simple!) ──────────────────
async fn handle_form(form: Form<PersonForm>) -> impl IntoResponse {
    let Form(form_data) = form;  // ← Extract without type annotation
    println!("🔍 Received form: {:#?}", form_data);

    let person: Person = form_data.into();  // ← Automatic conversion!
    println!("✅ Converted to domain: {person:#?}");

    (StatusCode::CREATED, json!(person).to_string())
}

// ── maud template (simple flat field names) ────────────
fn render_form(person: &Person) -> Markup {
    html! {
        form method="post" action="/submit" {
            label { (person.name.label) }
            input 
                type=(PersonForm::NAME_INPUT_TYPE)
                name="name_value"          // ← Simple flat name, no brackets!
                value=(person.name.value);
            br;
            
            label { (person.zip.label) }
            input 
                type=(person.zip.input_type)
                name="zip_value"           // ← Simple flat name, no brackets!
                value=(person.zip.value);
            br;
            
            label { "Age" }
            input 
                type="number" 
                name="age"                 // ← Direct mapping
                value=(person.age);
            br;
            
            button { "Save" }
        }
    }
}

// ── main ───────────────────────────────────────────────
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