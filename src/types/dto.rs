use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct TableRow {
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
    pub rows: Vec<TableRow>,
    pub column_count: usize,
    // pub column_names: Vec<&'static str>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DataType {
    NULL,
    INTEGER,
    FLOAT,
    TEXT,
    BLOB,
}

#[derive(Serialize, Deserialize, Debug)]
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