use crate::servidor::EstadoSistema;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub async fn tratar_dados_sensor(partes: &[&str], estado: &Arc<Mutex<EstadoSistema>>) -> String {
    let mut estado = estado.lock().await;
    let mut id_sensor = None;
    let mut tipo_sensor = None;

    for part in partes.iter().skip(1) {
        match *part {
            "ID" => {
                if let Some(valor) = partes.get(partes.iter().position(|&p| p == "ID").unwrap() + 1) {
                    id_sensor = Some(valor.to_string());
                }
            }
            "TEMPERATURA" => {
                tipo_sensor = Some("temperatura");
                if let Some(valor) = partes.get(partes.iter().position(|&p| p == "TEMPERATURA").unwrap() + 1) {
                    if let Ok(temp) = valor.parse::<f32>() {
                        estado.temperatura_interna = temp.clamp(-20.0, 50.0);
                    }
                }
            }
            "PORTA" => {
                tipo_sensor = Some("porta");
                if let Some(status) = partes.get(partes.iter().position(|&p| p == "PORTA").unwrap() + 1) {
                    estado.porta_aberta = status.eq_ignore_ascii_case("ABERTA");
                }
            }
            "ESTOQUE" => {
                tipo_sensor = Some("estoque");
                if let Some(percentual) = partes.get(partes.iter().position(|&p| p == "ESTOQUE").unwrap() + 1) {
                    if let Ok(p) = percentual.parse::<u8>() {
                        estado.nivel_estoque = p.clamp(0, 100);
                    }
                }
            }
            _ => {}
        }
    }

    let id_sensor = id_sensor.unwrap_or_else(|| Uuid::new_v4().to_string());

    match tipo_sensor {
        Some("temperatura") => estado.id_temperatura = id_sensor,
        Some("porta") => estado.id_porta = id_sensor,
        Some("estoque") => estado.id_estoque = id_sensor,
        _ => {}
    }

    "GERENCIADOR/1.0 200 OK\r\n\r\n".to_string()
}


// Handler para clientes
pub async fn handle_client_command(lines: &[&str], state: &Arc<Mutex<EstadoSistema>>) -> String {
    let mut state = state.lock().await;
    
    match lines.get(1) {
        Some(&"GET_STATUS") => format!(
            "\r\nGERENCIADOR/1.0 200 OK\r\n\
            TEMPERATURA {:.1} ID {}\r\n\
            PORTA {} ID {}\r\n\
            ESTOQUE {}% ID {}\r\n\
            REFRIGERADOR {}\r\n\
            ALARME {}\r\n\r\n",
            state.temperatura_interna,
            state.id_temperatura,
            if state.porta_aberta { "ABERTA" } else { "FECHADA" },
            state.id_porta,
            state.nivel_estoque,
            state.id_estoque,
            if state.refrigerador_ligado { "LIGADO" } else { "DESLIGADO" },
            if state.alarme_ativado { "ATIVADO" } else { "NORMAL" }
        ),
        Some(&"SET_LIMIT") => {
            if let Some(valor) = lines.get(2) {
                if let Ok(temp) = valor.parse::<f32>() {
                    state.temperatura_ideal = temp;
                    format!("GERENCIADOR/1.0 200 OK\r\nLIMITE: {:.1}\r\n\r\n", state.temperatura_ideal)
                } else {
                    "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
                }
            } else {
                "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
            }
        },
        None => "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string(),
        _ => "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
    }
}

pub async fn handle_actuator_command(lines: &[&str], state: &Arc<Mutex<EstadoSistema>>) -> String {
    let mut state = state.lock().await;
    
    for line in lines.iter().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        match parts[..] {
            ["REFRIGERADOR", status] => {
                state.refrigerador_ligado = status.eq_ignore_ascii_case("LIGAR");
                return "GERENCIADOR/1.0 200 OK\r\n\r\n".to_string();
            }
            ["LUZ", status] => {
                state.luz_ligada = status.eq_ignore_ascii_case("ACENDER");
                "GERENCIADOR/1.0 200 OK\r\n\r\n".to_string()
            }
            _ => "GERENCIADOR/1.0 400 ERROR\r\n\r\n".to_string()
        };
    }
    
    "GERENCIADOR/1.0 200 OK\r\n\r\n".to_string()
}