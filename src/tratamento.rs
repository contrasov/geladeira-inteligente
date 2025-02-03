use crate::servidor::EstadoSistema;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn handle_client_command(lines: &[&str], state: &Arc<Mutex<EstadoSistema>>) -> String {
    let mut state = state.lock().await;
    
    match lines.get(1) {
        Some(&"GET_STATUS") => format!(
            "\r\nGERENCIADOR/1.0 200 OK\r\n\r\n\
            TEMPERATURA: {:.1} | ID: {}\r\n\
            PORTA: {} | ID: {}\r\n\
            ESTOQUE: {}% | ID: {}\r\n\
            REFRIGERADOR: {} | ID: {}\r\n\
            LUZ: {} | ID: {}\r\n\
            ALARME: {}\r\n\r\n",
            state.temperatura_interna,
            state.id_temperatura,
            if state.porta_aberta { "ABERTA" } else { "FECHADA" },
            state.id_porta,
            state.nivel_estoque,
            state.id_estoque,
            if state.refrigerador_ligado { "LIGADO" } else { "DESLIGADO" },
            state.id_refrigerador,
            if state.luz_ligada { "ACESA" } else { "APAGADA" },
            state.id_luz,
            if state.alarme_ativado { "ATIVADO" } else { "NORMAL" }
        ),
        Some(&"SET_LIMITE") => {
            if let Some(valor) = lines.get(2) {
                if let Ok(temp) = valor.parse::<f32>() {
                    state.temperatura_ideal = temp;
                    format!("\r\nGERENCIADOR/1.0 200 OK\r\n\r\nLIMITE: {:.1}\r\n\r\n", state.temperatura_ideal)
                } else {
                    "\r\nGERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
                }
            } else {
                "\r\nGERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
            }
        },
        Some(&"SET_PORTA") => {
            if let Some(status) = lines.get(2) {
                let novo_estado = status.eq_ignore_ascii_case("ABERTA");
                state.porta_aberta = novo_estado;
                format!("\r\nGERENCIADOR/1.0 200 OK\r\nPORTA: {}\r\n\r\n", 
                    if novo_estado { "ABERTA" } else { "FECHADA" }
                )
            } else {
                "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
            }
        },
        None => "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string(),
        _ => "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
    }
}