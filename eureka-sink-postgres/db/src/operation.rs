use crate::sql_types::SqlType;
use std::collections::HashMap;

#[allow(dead_code)]
pub enum OperationType {
    Insert,
    Update,
    Delete,
}

pub struct Operation {
    schema_name: String,
    table_name: String,
    primary_key_column_name: String,
    op_type: OperationType,
    primary_key: SqlType,
    data: HashMap<String, SqlType>, // mapping data row from columns -> field
}

#[allow(dead_code)]
impl Operation {
    pub fn new(
        schema_name: String,
        table_name: String,
        primary_key_column_name: String,
        op_type: OperationType,
        primary_key: SqlType,
        data: HashMap<String, SqlType>,
    ) -> Self {
        Self {
            schema_name,
            table_name,
            primary_key_column_name,
            op_type,
            primary_key,
            data,
        }
    }

    // TODO: need to parse sql query in accordance with the data type
    pub fn build_query(&self) -> String {
        let query = match self.op_type {
            OperationType::Delete => {
                format!(
                    "DELETE FROM {}.{} WHERE {} = {}",
                    self.schema_name,
                    self.table_name,
                    self.primary_key_column_name,
                    self.primary_key.to_string()
                )
            }
            OperationType::Insert => {
                let mut keys = "".to_string();
                let mut values = "".to_string();

                // TODO: write this in a more idiomatic way
                self.data.iter().for_each(|(k, v)| {
                    keys.push_str(format!(",{}", k).as_str());
                    values.push_str(format!(",{}", v.to_string()).as_str());
                });
                // remove extra initial ','
                keys.remove(0); // ,col1,col2,col3,col4 -> col1,col2,col3,col4
                values.remove(0); //

                format!(
                    "INSERT INTO {}.{} ({}) VALUES ({})",
                    self.schema_name, self.table_name, keys, values
                )
            }
            OperationType::Update => {
                let mut updates = "".to_string();
                // TODO; write this more idiomatically
                self.data.iter().for_each(|(k, v)| {
                    updates.push_str(format!(",{}={}", k, v.to_string()).as_str())
                });
                // remove extra initial ','
                updates.remove(0);
                format!(
                    "UPDATE {}.{} SET {} WHERE {}={}",
                    self.schema_name,
                    self.table_name,
                    updates,
                    self.primary_key_column_name,
                    self.primary_key.to_string()
                )
            }
        };

        query
    }
}
