use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    api::models::{ApiResponse, ErrorResponse},
    conversion::{ConversionService, ConversionResult},
    errors::AstorError,
};

#[derive(Debug, Deserialize)]
pub struct ConvertRequest {
    pub from_currency: String,
    pub to_currency: String,
    pub amount: u64,
    pub max_slippage: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct ConvertResponse {
    pub result: ConversionResult,
    pub supported_currencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ExchangeRateQuery {
    pub from: String,
    pub to: String,
}

// Convert currency endpoint
pub async fn convert_currency(
    State(mut conversion_service): State<ConversionService>,
    Json(request): Json<ConvertRequest>,
) -> Result<Json<ApiResponse<ConvertResponse>>, (StatusCode, Json<ErrorResponse>)> {
    // Validate currencies
    if !conversion_service.is_supported_currency(&request.from_currency) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Unsupported source currency".to_string(),
                code: "INVALID_CURRENCY".to_string(),
            }),
        ));
    }

    if !conversion_service.is_supported_currency(&request.to_currency) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Unsupported target currency".to_string(),
                code: "INVALID_CURRENCY".to_string(),
            }),
        ));
    }

    match conversion_service
        .convert_with_fees(
            request.amount,
            &request.from_currency,
            &request.to_currency,
            request.max_slippage,
        )
        .await
    {
        Ok(result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(ConvertResponse {
                result,
                supported_currencies: conversion_service.get_supported_currencies().to_vec(),
            }),
            message: "Currency conversion completed successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "CONVERSION_FAILED".to_string(),
            }),
        )),
    }
}

// Get exchange rates endpoint
pub async fn get_exchange_rates(
    State(mut conversion_service): State<ConversionService>,
    Query(query): Query<ExchangeRateQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ErrorResponse>)> {
    // Refresh rates
    if let Err(e) = conversion_service.fetch_live_rates().await {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to fetch exchange rates: {}", e),
                code: "RATE_FETCH_FAILED".to_string(),
            }),
        ));
    }

    match conversion_service.get_exchange_rate_info(&query.from, &query.to) {
        Ok(rate_info) => Ok(Json(ApiResponse {
            success: true,
            data: Some(serde_json::json!({
                "rate": rate_info.rate,
                "bid": rate_info.bid,
                "ask": rate_info.ask,
                "timestamp": rate_info.timestamp,
                "source": rate_info.source,
                "volatility": rate_info.volatility,
                "daily_change": rate_info.daily_change
            })),
            message: "Exchange rate retrieved successfully".to_string(),
        })),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: e.to_string(),
                code: "RATE_NOT_FOUND".to_string(),
            }),
        )),
    }
}

// Get supported currencies endpoint
pub async fn get_supported_currencies(
    State(conversion_service): State<ConversionService>,
) -> Json<ApiResponse<Vec<String>>> {
    Json(ApiResponse {
        success: true,
        data: Some(conversion_service.get_supported_currencies().to_vec()),
        message: "Supported currencies retrieved successfully".to_string(),
    })
}
