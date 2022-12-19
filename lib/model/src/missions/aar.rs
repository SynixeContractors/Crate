//! After action report

use std::{collections::HashMap, fmt::Display};

use regex::Regex;
use time::Date;

#[derive(Debug)]
/// After action report
pub struct Aar {
    /// Mission pretty name
    mission: String,
    /// Mission date
    date: Date,
    /// Contractors in attendance
    contractors: Vec<String>,
    /// Outcome of the mission
    outcome: Outcome,
    /// Payment for the mission
    payment: Payment,
}

impl Aar {
    /// Parses an AAR from a discord message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message is not a valid AAR.
    pub fn from_message(content: &str) -> Result<Self, String> {
        let content = content.trim_matches('`');
        let lower = content.to_lowercase();
        let Ok(regex) = Regex::new(r"(?m)(\d+)(?:.+?)(no|light|medium|heavy)") else {
            return Err("Could not compile regex.".to_string());
        };
        let mut lines = HashMap::new();
        for line in content.lines() {
            if line.is_empty() || !line.contains(": ") {
                continue;
            }
            if let Some(split) = line.split_once(": ") {
                lines.insert(split.0.to_lowercase(), split.1);
            }
        }
        let mission_type = {
            if lines.contains_key("contract") {
                "contract"
            } else if lines.contains_key("subcontract") {
                "subcontract"
            } else if lines.contains_key("training") {
                "training"
            } else if lines.contains_key("special") {
                "special"
            } else {
                return Err("Could not determine mission type. Valid types are Contract, Subcontract, Training, and Special.".to_string());
            }
        };
        let Some(mission_name) = lines.get(mission_type) else { return Err(format!("Could not find mission name for mission type {mission_type}")) };

        let Some(date) = lines.get("date") else { return Err("Could not find date.".to_string()) };
        let Ok(date) = Date::parse(date, time::macros::format_description!("[year]-[month]-[day]")) else {
            return Err(format!("Could not parse date: {date}. Make sure it's in the format YYYY-MM-DD."));
        };

        let contractors = {
            let Some(contractors) = lines.get("contractors") else { return Err("Could not find contractors.".to_string()) };
            contractors
                .split(", ")
                .map(std::string::ToString::to_string)
                .collect()
        };

        let result = regex.captures_iter(&lower);
        let mut payment = Payment::default();
        for mat in result {
            let Some(mat1) = mat.get(1) else { return Err("Could not find payment amount.".to_string()) };
            let Ok(amount) = mat1.as_str().parse::<i32>() else {
                return Err(format!("Could not parse payment amount: {}", mat1.as_str()));
            };
            let Some(mat2) = mat.get(2) else { return Err("Could not find payment type.".to_string()) };
            let kind = mat2.as_str();
            match kind {
                "no" => payment.no_combat = amount,
                "light" => payment.light_combat = amount,
                "medium" => payment.medium_combat = amount,
                "heavy" => payment.heavy_combat = amount,
                _ => return Err(format!("Unknown payment type: {kind}")),
            }
        }

        let outcome = {
            if lower.contains("operation successful")
                || lower.contains("operation completed")
                || lower.contains("operation success")
            {
                Outcome::Success
            } else if lower.contains("operation partial") {
                Outcome::Partial
            } else if lower.contains("operation failure") || lower.contains("operation failed") {
                Outcome::Failure
            } else {
                return Err("Could not determine mission outcome. Valid outcomes are Operation Successful, Operation Partial Success, and Operation Failure.".to_string());
            }
        };

        Ok(Self {
            mission: (*mission_name).to_string(),
            date,
            contractors,
            outcome,
            payment,
        })
    }

    #[must_use]
    /// Returns the mission pretty name.
    pub fn mission(&self) -> &str {
        &self.mission
    }

    #[must_use]
    /// Returns the mission date.
    pub const fn date(&self) -> Date {
        self.date
    }

    #[must_use]
    /// Returns the contractors in attendance.
    pub fn contractors(&self) -> &[String] {
        &self.contractors
    }

    #[must_use]
    /// Returns the outcome of the mission.
    pub const fn outcome(&self) -> &Outcome {
        &self.outcome
    }

    #[must_use]
    /// Returns the payment for the mission.
    pub const fn payment(&self) -> &Payment {
        &self.payment
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    /// Calculates the contractor payment for the mission.
    pub fn contractor_payment(&self, payment_type: PaymentType) -> i32 {
        let mut payment = 0f32;
        payment += self.payment.no_combat as f32 / 60f32 * payment_type.no_combat() as f32;
        payment += self.payment.light_combat as f32 / 60f32 * payment_type.light_combat() as f32;
        payment += self.payment.medium_combat as f32 / 60f32 * payment_type.medium_combat() as f32;
        payment += self.payment.heavy_combat as f32 / 60f32 * payment_type.heavy_combat() as f32;
        payment *= self.outcome.contractor_multiplier();
        payment as i32
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    /// Calculates the employer payment for the mission.
    pub fn employer_payment(&self, payment_type: PaymentType) -> i32 {
        let mut payment = 0f32;
        payment += self.payment.total() as f32 / 60f32 * payment_type.employer() as f32;
        payment *= self.outcome.employer_multiplier();
        (payment as i32).max(20_000)
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    /// Show the math for the payments
    /// Example:
    /// ```
    /// No Combat  | 420/h x 60 = 420
    /// Light      | 490/h x 30 = 245
    /// Heavy      | 700/h x 30 = 350
    /// Partial    | x0.8       = 812
    ///
    /// Group      | 59800/h x 120 = 119600
    /// Partial    | x0.5       = 59800
    /// ```
    pub fn show_math(&self, payment_type: PaymentType) -> String {
        let mut math = String::new();
        if self.payment.no_combat > 0 {
            math.push_str(&format!(
                "No Combat  | {:03}/h x {:02} = {}\n",
                payment_type.no_combat(),
                self.payment.no_combat,
                self.payment.no_combat as f32 / 60f32 * payment_type.no_combat() as f32
            ));
        }
        if self.payment.light_combat > 0 {
            math.push_str(&format!(
                "Light      | {:03}/h x {:02} = {}\n",
                payment_type.light_combat(),
                self.payment.light_combat,
                self.payment.light_combat as f32 / 60f32 * payment_type.light_combat() as f32
            ));
        }
        if self.payment.medium_combat > 0 {
            math.push_str(&format!(
                "Medium     | {:03}/h x {:02} = {}\n",
                payment_type.medium_combat(),
                self.payment.medium_combat,
                self.payment.medium_combat as f32 / 60f32 * payment_type.medium_combat() as f32
            ));
        }
        if self.payment.heavy_combat > 0 {
            math.push_str(&format!(
                "Heavy      | {:03}/h x {:02} = {}\n",
                payment_type.heavy_combat(),
                self.payment.heavy_combat,
                self.payment.heavy_combat as f32 / 60f32 * payment_type.heavy_combat() as f32
            ));
        }
        math.push_str(&format!(
            "{}    | x{:.1}       = ${}\n",
            self.outcome,
            self.outcome.contractor_multiplier(),
            bootstrap::format::money(self.contractor_payment(payment_type))
        ));
        math.push('\n');
        math.push_str(&format!(
            "Group      | {}/h x {:03} = {}\n",
            payment_type.employer(),
            self.payment.total(),
            self.payment.total() as f32 / 60f32 * payment_type.employer() as f32
        ));
        math.push_str(&format!(
            "{}    | x{:.1}          = ${}\n",
            self.outcome,
            self.outcome.employer_multiplier(),
            bootstrap::format::money(self.employer_payment(payment_type))
        ));
        if self.outcome() == &Outcome::Failure {
            math.push_str("Base      | $20,000");
        }
        math
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
/// The outcome of a mission.
pub enum Outcome {
    /// The mission was a success.
    Success,
    /// The mission was a partial success.
    Partial,
    /// The mission was a failure.
    Failure,
}

impl Outcome {
    #[must_use]
    /// Returns the contractor multiplier for the outcome.
    pub const fn contractor_multiplier(&self) -> f32 {
        match self {
            Self::Success => 1.0,
            Self::Partial => 0.8,
            Self::Failure => 0.6,
        }
    }

    #[must_use]
    /// Returns the employer multiplier for the outcome.
    pub const fn employer_multiplier(&self) -> f32 {
        match self {
            Self::Success => 1.0,
            Self::Partial => 0.5,
            Self::Failure => 0.0,
        }
    }
}

impl Display for Outcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::Partial => write!(f, "Partial"),
            Self::Failure => write!(f, "Failure"),
        }
    }
}

#[derive(Debug, Default)]
/// The payment for a mission.
pub struct Payment {
    /// The duration in minutes of no combat.
    no_combat: i32,
    /// The duration in minutes of light combat.
    light_combat: i32,
    /// The duration in minutes of medium combat.
    medium_combat: i32,
    /// The duration in minutes of heavy combat.
    heavy_combat: i32,
}

impl Payment {
    #[must_use]
    /// Returns the total payment for the mission.
    pub const fn total(&self) -> i32 {
        self.no_combat + self.light_combat + self.medium_combat + self.heavy_combat
    }
}

#[derive(Debug, Clone, Copy)]
/// Payments rates for a mission.
/// !! Do not change the order of these values !!
pub enum PaymentType {
    /// Protection in a low risk area.
    ProtectionLowRisk,
    /// Protection in a high risk area.
    ProtectionHighRisk,
    /// Logistics
    Logistics,
    /// Recon
    Recon,
    /// Offensive
    Offensive,
    /// Defensive
    Defensive,
    /// Support
    Support,
    /// Security
    Security,
    /// Smash and Grab
    SmashGrab,
}

impl PaymentType {
    #[must_use]
    /// Returns a map of the payment types and their values.
    pub fn as_choices() -> Vec<(String, i32)> {
        vec![
            (
                "Protection (Low Risk)".to_string(),
                Self::ProtectionLowRisk as i32,
            ),
            (
                "Protection (High Risk)".to_string(),
                Self::ProtectionHighRisk as i32,
            ),
            ("Logistics".to_string(), Self::Logistics as i32),
            ("Recon".to_string(), Self::Recon as i32),
            ("Offensive".to_string(), Self::Offensive as i32),
            ("Defensive".to_string(), Self::Defensive as i32),
            ("Support".to_string(), Self::Support as i32),
            ("Security".to_string(), Self::Security as i32),
            ("Smash and Grab".to_string(), Self::SmashGrab as i32),
        ]
    }

    #[must_use]
    /// Returns the payment type from the given value.
    pub const fn from_i32(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::ProtectionLowRisk),
            1 => Some(Self::ProtectionHighRisk),
            2 => Some(Self::Logistics),
            3 => Some(Self::Recon),
            4 => Some(Self::Offensive),
            5 => Some(Self::Defensive),
            6 => Some(Self::Support),
            7 => Some(Self::Security),
            8 => Some(Self::SmashGrab),
            _ => None,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for no combat.
    pub const fn no_combat(&self) -> i32 {
        match self {
            Self::ProtectionLowRisk => 420,
            Self::ProtectionHighRisk => 490,
            Self::Logistics => 280,
            Self::Recon => 280,
            Self::Offensive => 350,
            Self::Defensive => 420,
            Self::Support => 280,
            Self::Security => 350,
            Self::SmashGrab => 420,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for light combat.
    pub const fn light_combat(&self) -> i32 {
        match self {
            Self::ProtectionLowRisk => 490,
            Self::ProtectionHighRisk => 560,
            Self::Logistics => 350,
            Self::Recon => 350,
            Self::Offensive => 420,
            Self::Defensive => 490,
            Self::Support => 350,
            Self::Security => 420,
            Self::SmashGrab => 490,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for medium combat.
    pub const fn medium_combat(&self) -> i32 {
        match self {
            Self::ProtectionLowRisk => 560,
            Self::ProtectionHighRisk => 700,
            Self::Logistics => 420,
            Self::Recon => 420,
            Self::Offensive => 490,
            Self::Defensive => 560,
            Self::Support => 420,
            Self::Security => 490,
            Self::SmashGrab => 350,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for heavy combat.
    pub const fn heavy_combat(&self) -> i32 {
        match self {
            Self::ProtectionLowRisk => 700,
            Self::ProtectionHighRisk => 840,
            Self::Logistics => 560,
            Self::Recon => 560,
            Self::Offensive => 560,
            Self::Defensive => 630,
            Self::Support => 560,
            Self::Security => 560,
            Self::SmashGrab => 210,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for the employer
    pub const fn employer(&self) -> i32 {
        match self {
            Self::ProtectionLowRisk => 59800,
            Self::ProtectionHighRisk => 89700,
            Self::Logistics => 44850,
            Self::Recon => 44850,
            Self::Offensive => 59800,
            Self::Defensive => 67275,
            Self::Support => 44850,
            Self::Security => 59800,
            Self::SmashGrab => 44850,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aar() {
        let aar = Aar::from_message(
            r#"```Contract: Pusherman
Date: 2022-12-17
OL: Jake King
ELs: Carson Sering (KIA), Chaplain Yi (After Casualty), John Lamb (KIA)

Contractors: Jake King, Brett Harrison, Nathanial Greene, Carson Sering, Sean Miles, Chaplain Yi, Matías Jackson, John Brown, John Lamb
Assets Deployed: 2x Arcadian
Assets Lost: None
Casualties: John Lamb, Carson Sering

AAR: Contractors were tasked with destroying Cartel Assets on the island nation of Tanoa. Contractors began by assaulting the airfield. There was a handful of armed cartel members present at the airfield, which were taken out in the assault. A Cartel owned C-130, Mohawk Transport Helicopter, and Fuel Truck was destroyed. The ATC tower was resistant to demolition charges, so one was set in the tower itself to destroy the electronics inside of the tower itself. From the airfield, contractors regrouped at the FOB, and then began to make their way to the town of Tavu. A siege of the town began to neutralize the cartel presence within the town. In the early moments of contact with the cartel, both FTL's were KIA. After regrouping and triaging patients, the remaining cartel members were neutralized. 7 cartel owned RHIB's were destroyed, 2 cartel owned vans were destroyed, and 5 crates of cartel product were destroyed. Contractors successfully exfil'd back to the fob.

Operation Successful.

Payment request: 60 No Combat 30 Light Combat 45 Medium Combat 15 Heavy Combat```"#,
        );
        assert!(aar.is_ok());
        println!("{}", aar.unwrap().show_math(PaymentType::Defensive));
    }
}
