use crate::state::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn process_request(request: &str, state: Arc<Mutex<AppState>>) -> String {
    let mut sensores = state.sensores.lock().await;
    let mut atuadores = state.atuadores.lock().await;

    if request.starts_with("GET /sensor/temperatura") {
        if let Some(temp) = sensores.get("S1") {
            return format!("MANAGER/1.0 200 OK\r\nContent-Length: {}\r\n\r\nS1 TEMPERATURA {}", temp.len(), temp);
        }
        return "MANAGER/1.0 404 NOT_FOUND\r\nContent-Length: 15\r\n\r\nSENSOR NÃO ENCONTRADO".to_string();
    }
    
    if request.starts_with("POST /sensor/") {
        let parts: Vec<&str> = request.split_whitespace().collect();
        if parts.len() >= 3 {
            let sensor_id = parts[2];
            let valor = parts[3];
            sensores.insert(sensor_id.to_string(), valor.to_string());
            return format!("MANAGER/1.0 200 OK\r\nContent-Length: 22\r\n\r\nSENSOR {} ATUALIZADO", sensor_id);
        }
    }

    if request.starts_with("CMD A1 LIGAR_REFRIGERADOR") {
        atuadores.insert("A1".to_string(), "LIGADO".to_string());
        return "MANAGER/1.0 200 OK\r\nContent-Length: 27\r\n\r\nCMD A1 LIGAR_REFRIGERADOR".to_string();
    }

    if request.starts_with("CMD A2 LIGAR_LUZ") {
        atuadores.insert("A2".to_string(), "ACESA".to_string());
        return "MANAGER/1.0 200 OK\r\nContent-Length: 19\r\n\r\nCMD A2 LIGAR_LUZ".to_string();
    }

    "MANAGER/1.0 400 BAD_REQUEST\r\nContent-Length: 17\r\n\r\nERRO COMANDO_INVÁLIDO".to_string()
}
