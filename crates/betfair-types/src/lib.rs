//! # Betfair Types library

pub use rust_decimal;

pub mod types {
    #![allow(clippy::all)]
    include!(concat!(env!("OUT_DIR"), "/mod.rs"));
}

pub mod keep_alive {
    pub use crate::shared::*;
}

pub mod logout {
    pub use crate::shared::*;
}

pub mod bot_login {
    use redact::Secret;
    use serde::{Deserialize, Deserializer, Serialize};
    use serde_json::Value;

    #[derive(Debug)]
    pub struct BotLoginResponse(pub Result<SuccessResponse, LoginError>);

    impl std::ops::Deref for BotLoginResponse {
        type Target = Result<SuccessResponse, LoginError>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SuccessResponse {
        #[serde(serialize_with = "redact::expose_secret")]
        pub session_token: redact::Secret<String>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum LoginError {
        #[doc = "the username or password are invalid"]
        #[serde(rename = "INVALID_USERNAME_OR_PASSWORD")]
        InvalidUsernameOrPassword,

        #[doc = "the account was just locked"]
        #[serde(rename = "ACCOUNT_NOW_LOCKED")]
        AccountNowLocked,

        #[doc = "the account is already locked"]
        #[serde(rename = "ACCOUNT_ALREADY_LOCKED")]
        AccountAlreadyLocked,

        #[doc = "pending authentication"]
        #[serde(rename = "PENDING_AUTH")]
        PendingAuth,

        #[doc = "Telbet terms and conditions rejected"]
        #[serde(rename = "TELBET_TERMS_CONDITIONS_NA")]
        TelbetTermsConditionsNa,

        #[doc = "duplicate cards"]
        #[serde(rename = "DUPLICATE_CARDS")]
        DuplicateCards,

        #[doc = "the user has entered wrong the security answer 3 times"]
        #[serde(rename = "SECURITY_QUESTION_WRONG_3X")]
        SecurityQuestionWrong3x,

        #[doc = "KYC suspended"]
        #[serde(rename = "KYC_SUSPEND")]
        KycSuspend,

        #[doc = "the account is suspended"]
        #[serde(rename = "SUSPENDED")]
        Suspended,

        #[doc = "the account is closed"]
        #[serde(rename = "CLOSED")]
        Closed,

        #[doc = "the account has been self-excluded"]
        #[serde(rename = "SELF_EXCLUDED")]
        SelfExcluded,

        #[doc = "the DK regulator cannot be accessed due to some internal problems in the system behind or in at regulator; timeout cases included."]
        #[serde(rename = "INVALID_CONNECTIVITY_TO_REGULATOR_DK")]
        InvalidConnectivityToRegulatorDk,

        #[doc = "the user identified by the given credentials is not authorized in the DK's jurisdictions due to the regulators' policies. Ex: the user for which this session should be created is not allowed to act(play, bet) in the DK's jurisdiction."]
        #[serde(rename = "NOT_AUTHORIZED_BY_REGULATOR_DK")]
        NotAuthorizedByRegulatorDk,

        #[doc = "the IT regulator cannot be accessed due to some internal problems in the system behind or in at regulator; timeout cases included."]
        #[serde(rename = "INVALID_CONNECTIVITY_TO_REGULATOR_IT")]
        InvalidConnectivityToRegulatorIt,

        #[doc = "the user identified by the given credentials is not authorized in the IT's jurisdictions due to the regulators' policies. Ex: the user for which this session should be created is not allowed to act(play, bet) in the IT's jurisdiction."]
        #[serde(rename = "NOT_AUTHORIZED_BY_REGULATOR_IT")]
        NotAuthorizedByRegulatorIt,

        #[doc = "the account is restricted due to security concerns"]
        #[serde(rename = "SECURITY_RESTRICTED_LOCATION")]
        SecurityRestrictedLocation,

        #[doc = "the account is accessed from a location where betting is restricted"]
        #[serde(rename = "BETTING_RESTRICTED_LOCATION")]
        BettingRestrictedLocation,

        #[doc = "Trading Master Account"]
        #[serde(rename = "TRADING_MASTER")]
        TradingMaster,

        #[doc = "Suspended Trading Master Account"]
        #[serde(rename = "TRADING_MASTER_SUSPENDED")]
        TradingMasterSuspended,

        #[doc = "Agent Client Master"]
        #[serde(rename = "AGENT_CLIENT_MASTER")]
        AgentClientMaster,

        #[doc = "Suspended Agent Client Master"]
        #[serde(rename = "AGENT_CLIENT_MASTER_SUSPENDED")]
        AgentClientMasterSuspended,

        #[doc = "Danish authorization required"]
        #[serde(rename = "DANISH_AUTHORIZATION_REQUIRED")]
        DanishAuthorizationRequired,

        #[doc = "Spain migration required"]
        #[serde(rename = "SPAIN_MIGRATION_REQUIRED")]
        SpainMigrationRequired,

        #[doc = "Denmark migration required"]
        #[serde(rename = "DENMARK_MIGRATION_REQUIRED")]
        DenmarkMigrationRequired,

        #[doc = "The latest Spanish terms and conditions version must be accepted. You must login to the website to accept the new conditions."]
        #[serde(rename = "SPANISH_TERMS_ACCEPTANCE_REQUIRED")]
        SpanishTermsAcceptanceRequired,

        #[doc = "The latest Italian contract version must be accepted. You must login to the website to accept the new conditions."]
        #[serde(rename = "ITALIAN_CONTRACT_ACCEPTANCE_REQUIRED")]
        ItalianContractAcceptanceRequired,

        #[doc = "Certificate required or certificate present but could not authenticate with it. Please check that the correct file path is specified and ensure you are entering the correct password."]
        #[serde(rename = "CERT_AUTH_REQUIRED")]
        CertAuthRequired,

        #[doc = "Change password required"]
        #[serde(rename = "CHANGE_PASSWORD_REQUIRED")]
        ChangePasswordRequired,

        #[doc = "Personal message required for the user"]
        #[serde(rename = "PERSONAL_MESSAGE_REQUIRED")]
        PersonalMessageRequired,

        #[doc = "The latest international terms and conditions must be accepted prior to logging in."]
        #[serde(rename = "INTERNATIONAL_TERMS_ACCEPTANCE_REQUIRED")]
        InternationalTermsAcceptanceRequired,

        #[doc = "This account has not opted in to log in with the email"]
        #[serde(rename = "EMAIL_LOGIN_NOT_ALLOWED")]
        EmailLoginNotAllowed,

        #[doc = "There is more than one account with the same credential"]
        #[serde(rename = "MULTIPLE_USERS_WITH_SAME_CREDENTIAL")]
        MultipleUsersWithSameCredential,

        #[doc = "The account must undergo password recovery to reactivate via https://identitysso.betfair.com/view/recoverpassword"]
        #[serde(rename = "ACCOUNT_PENDING_PASSWORD_CHANGE")]
        AccountPendingPasswordChange,

        #[doc = "The limit for successful login requests per minute has been exceeded. New login attempts will be banned for 20 minutes"]
        #[serde(rename = "TEMPORARY_BAN_TOO_MANY_REQUESTS")]
        TemporaryBanTooManyRequests,

        #[doc = "You must login to the website to accept the new conditions"]
        #[serde(rename = "ITALIAN_PROFILING_ACCEPTANCE_REQUIRED")]
        ItalianProfilingAcceptanceRequired,

        #[doc = "You are attempting to login to the Betfair Romania domain with a non .ro account."]
        #[serde(rename = "AUTHORIZED_ONLY_FOR_DOMAIN_RO")]
        AuthorizedOnlyForDomainRo,

        #[doc = "You are attempting to login to the Betfair Swedish domain with a non .se account."]
        #[serde(rename = "AUTHORIZED_ONLY_FOR_DOMAIN_SE")]
        AuthorizedOnlyForDomainSe,

        #[doc = "You must provided your Swedish National identifier via Betfair.se before proceeding."]
        #[serde(rename = "SWEDEN_NATIONAL_IDENTIFIER_REQUIRED")]
        SwedenNationalIdentifierRequired,

        #[doc = "You must provided your Swedish bank id via Betfair.se before proceeding."]
        #[serde(rename = "SWEDEN_BANK_ID_VERIFICATION_REQUIRED")]
        SwedenBankIdVerificationRequired,

        #[doc = "You must login to https://www.betfair.com to provide the missing information."]
        #[serde(rename = "ACTIONS_REQUIRED")]
        ActionsRequired,

        #[doc = "There is a problem with the data validity contained within the request. Please check that the request (including headers) is in the correct format,"]
        #[serde(rename = "INPUT_VALIDATION_ERROR")]
        InputValidationError,
    }

    impl<'de> Deserialize<'de> for BotLoginResponse {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = Value::deserialize(deserializer)?;

            // Check if it's a success response
            if let Some(session_token) = value.get("sessionToken").and_then(|v| v.as_str()) {
                let success_response = SuccessResponse {
                    session_token: Secret::new(session_token.to_owned()),
                };
                return Ok(BotLoginResponse(Ok(success_response)))
            }

            // If not success, parse as an error
            if let Some(err) = value.get("loginStatus") {
                let login_error = LoginError::deserialize(err).map_err(serde::de::Error::custom)?;
                return Ok(BotLoginResponse(Err(login_error)))
            }

            Err(serde::de::Error::custom("invalid response"))
        }
    }
}

mod shared {

    use serde::{Deserialize, Deserializer};
    use serde_json::Value;

    #[derive(Debug, Deserialize)]
    pub struct SuccessResponse {
        pub token: redact::Secret<String>,
        pub product: redact::Secret<String>,
    }

    #[derive(Debug)]
    pub struct Response(pub Result<SuccessResponse, ErrorValues>);

    impl AsRef<Result<SuccessResponse, ErrorValues>> for Response {
        fn as_ref(&self) -> &Result<SuccessResponse, ErrorValues> {
            &self.0
        }
    }

    impl std::ops::Deref for Response {
        type Target = Result<SuccessResponse, ErrorValues>;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::ops::DerefMut for Response {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl<'de> Deserialize<'de> for Response {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let value = Value::deserialize(deserializer)?;

            // Check if it's a success response
            if let Some(status) = value.get("status").and_then(|v| v.as_str()) {
                match status {
                    "SUCCESS" => {
                        let success_response = SuccessResponse {
                            token: redact::Secret::new(
                                value
                                    .get("token")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| serde::de::Error::custom("invalid response"))?
                                    .to_owned(),
                            ),
                            product: redact::Secret::new(
                                value
                                    .get("product")
                                    .and_then(|v| v.as_str())
                                    .ok_or_else(|| serde::de::Error::custom("invalid response"))?
                                    .to_owned(),
                            ),
                        };
                        return Ok(Response(Ok(success_response)))
                    }
                    "FAIL" => {
                        if let Some(error) = value.get("error") {
                            let login_error = ErrorValues::deserialize(error)
                                .map_err(serde::de::Error::custom)?;
                            return Ok(Response(Err(login_error)))
                        }
                    }
                    _ => {}
                }
            }

            Err(serde::de::Error::custom("invalid response"))
        }
    }
    #[derive(Debug, Deserialize)]
    pub enum StatusValues {
        #[serde(rename = "SUCCESS")]
        Success,
        #[serde(rename = "FAIL")]
        Failure,
    }

    #[derive(Debug, Deserialize)]
    pub enum ErrorValues {
        #[serde(rename = "INPUT_VALIDATION_ERROR")]
        InputValidation,
        #[serde(rename = "INVALID_SESSION_INFORMATION")]
        InvalidSessionInformation,
        #[serde(rename = "NO_SESSION")]
        NoSession,
    }
}
