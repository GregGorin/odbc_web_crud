// extern crate odbc;
use odbc::*;
use serde::{Deserialize, Serialize};
// use std::sync::Arc;
// use std::sync::Mutex;

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub odbc_source: String,
    pub sql_text: String,
    pub data_set: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JobResult {
    success: bool,
    message: String,
    data_set: Vec<Vec<String>>,
}

pub fn execute_job(job: Job) -> JobResult {
    let mut jod_result = JobResult {
        success: true,
        message: String::from("Success"),
        data_set: vec![],
    };

    match connect_and_execute(job) {
        Ok(data_set) => {
            jod_result.data_set = data_set;
            jod_result
        }
        Err(diag) => {
            jod_result.message = diag.to_string();
            jod_result.success = false;
            jod_result
        }
    }
}

fn make_command(data_line: &Vec<String>, sql_text: &str) -> String {
    let mut command = String::from(sql_text);

    for index in 0..data_line.len() {
        let column_name = format!("#{}", index + 1);
        command = command.replace(&column_name, &data_line[index]);
    }

    command
}

fn connect_and_execute(job: Job) -> std::result::Result<Vec<Vec<String>>, DiagnosticRecord> {
    let env: Environment<odbc_safe::Odbc3> = create_environment_v3().map_err(|e| e.unwrap())?;
    let conn: Connection<'_, odbc_safe::AutocommitOn> =
        env.connect_with_connection_string(&job.odbc_source)?;

    let mut data_set = Vec::<Vec<String>>::new();

    for data_line in job.data_set {
        let sql_command = make_command(&data_line, &job.sql_text);
        // println!("Command: {}", sql_command);
        let stmt = Statement::with_parent(&conn)?;
        match stmt.exec_direct(&sql_command)? {
            Data(mut stmt) => {
                let cols = stmt.num_result_cols()?;
                while let Some(mut cursor) = stmt.fetch()? {
                    let mut data_set_line = Vec::<String>::new();
                    for i in 1..(cols + 1) {
                        data_set_line.push(match cursor.get_data::<&str>(i as u16)? {
                            Some(val) => val.to_owned(),
                            None => String::from(" NULL"),
                        });
                    }
                    data_set.push(data_set_line);
                }
            }
            NoData(_) => (),
        }
    }

    Ok(data_set)
}
