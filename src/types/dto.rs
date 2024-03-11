use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct TableRowDTO {
    pub columns: Vec<ColumnData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryDTO {
    pub query: String,
    pub query_type: QueryType,
    pub params: Vec<ColumnData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResponseDTO {
    pub status: String,
    pub rows_affected: usize,
    pub rows: Vec<TableRowDTO>,
    pub column_count: usize,
    pub column_names: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DataType {
    NULL,
    INTEGER,
    FLOAT,
    TEXT,
    BLOB,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ColumnData {
    pub data: String,
    pub data_type: DataType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum QueryType {
    EXEC,
    QUERY,
    INSERT
}