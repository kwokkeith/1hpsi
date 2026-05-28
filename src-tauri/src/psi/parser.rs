use crate::psi::models::PsiResponse;

pub fn parse_psi_response(json: &str) -> Result<PsiResponse, serde_json::Error> {
    serde_json::from_str(json)
}
