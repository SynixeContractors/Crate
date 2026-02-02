//! After action report

use std::{
    collections::{HashMap, HashSet},
    fmt::{Display, Write},
    str::FromStr,
};

use regex::Regex;
use time::Date;

use super::MissionType;

#[derive(Debug)]
/// After action report
pub struct Aar {
    /// AAR content
    content: String,
    /// Mission type
    typ: MissionType,
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
    #[must_use]
    pub fn clean_content(content: &str) -> String {
        content
            .trim()
            .replace("\u{2068}", "")
            .replace("\u{2069}", "")
    }

    #[allow(clippy::too_many_lines)]
    /// Parses an AAR from a discord message.
    ///
    /// # Errors
    ///
    /// Returns an error if the message is not a valid AAR.
    pub fn from_message(content: &str) -> Result<Self, String> {
        let content = strip_ansi_escapes::strip_str(content.replace("```ansi", ""));
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
        let Some(mission_name) = lines.get(mission_type) else {
            return Err(format!(
                "Could not find mission name for mission type {mission_type}"
            ));
        };

        let Some(date) = lines.get("date") else {
            return Err("Could not find date.".to_string());
        };
        let Ok(date) = Date::parse(
            date,
            time::macros::format_description!("[year]-[month]-[day]"),
        ) else {
            return Err(format!(
                "Could not parse date: {date}. Make sure it's in the format YYYY-MM-DD."
            ));
        };

        let mut contractors: Vec<String> = {
            let Some(contractors) = lines.get("contractors") else {
                return Err("Could not find contractors.".to_string());
            };
            contractors
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect()
        };

        let unique = contractors.iter().collect::<HashSet<_>>();
        if unique.len() != contractors.len() {
            contractors.sort();
            let mut duplicates = Vec::new();
            for i in 0..contractors.len() - 2 {
                if contractors[i] == contractors[i + 1] {
                    duplicates.push(contractors[i].clone());
                }
            }
            if !duplicates.is_empty() {
                return Err(format!(
                    "Duplicate contractors: {duplicates}",
                    duplicates = duplicates.join(", ")
                ));
            }
        }

        let result = regex.captures_iter(&lower);
        let mut payment = Payment::default();
        for mat in result {
            let Some(mat1) = mat.get(1) else {
                return Err("Could not find payment amount.".to_string());
            };
            let Ok(amount) = mat1.as_str().parse::<i32>() else {
                return Err(format!("Could not parse payment amount: {}", mat1.as_str()));
            };
            let Some(mat2) = mat.get(2) else {
                return Err("Could not find payment type.".to_string());
            };
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
            content: content.to_string(),
            typ: MissionType::from_str(mission_type)?,
            mission: (*mission_name).to_string(),
            date,
            contractors,
            outcome,
            payment,
        })
    }

    #[must_use]
    /// Returns the AAR content.
    pub fn content(&self) -> &str {
        &self.content
    }

    #[must_use]
    /// Returns the mission type.
    pub const fn typ(&self) -> MissionType {
        self.typ
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
    pub fn contractor_payment(&self, payment_type: PaymentType, reputation: f32) -> i32 {
        let mut payment = 0f32;
        payment += self.payment.no_combat as f32 / 60f32 * payment_type.no_combat() as f32;
        payment += self.payment.light_combat as f32 / 60f32 * payment_type.light_combat() as f32;
        payment += self.payment.medium_combat as f32 / 60f32 * payment_type.medium_combat() as f32;
        payment += self.payment.heavy_combat as f32 / 60f32 * payment_type.heavy_combat() as f32;
        payment += reputation.clamp(-300f32, 300f32) / 120f32 * self.payment.total() as f32;
        payment *= self.outcome.contractor_multiplier();
        payment as i32
    }

    #[must_use]
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    /// Calculates the employer payment for the mission.
    pub fn employer_payment(&self, payment_type: PaymentType, reputation: f32) -> i32 {
        let mut payment = 0f32;
        payment += self.payment.total() as f32 / 60f32 * payment_type.employer() as f32;
        payment += reputation * 200f32 / 120f32 * self.payment.total() as f32;
        payment *= self.outcome.employer_multiplier();
        (payment as i32).max(20_000)
    }

    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        clippy::too_many_lines
    )]
    /// Show the math for the payments
    /// Example:
    /// ```txt
    /// No Combat  | 420/h x 60  = $420
    /// Light      | 490/h x 30  = $245
    /// Heavy      | 700/h x 30  = $350
    /// Reputation | 099/h x 120 = $181
    /// Partial    | x0.8        = $956
    ///
    /// Group      | 59800/h x 120 = $119,600
    /// Reputation | 09945/h x 120 = $19,890
    /// Partial    | x0.5          = $69,745
    /// ```
    pub fn show_math(&self, payment_type: PaymentType, reputation: f32) -> String {
        let mut math = String::new();
        if self.payment.no_combat > 0 {
            writeln!(
                math,
                "No Combat  |  {:03}/h x {:03} = {}",
                payment_type.no_combat(),
                self.payment.no_combat,
                bootstrap::format::money(
                    (self.payment.no_combat as f32 / 60f32 * payment_type.no_combat() as f32).ceil()
                        as i32,
                    true
                )
            )
            .expect("should be able to write to string");
        }
        if self.payment.light_combat > 0 {
            writeln!(
                math,
                "Light      |  {:03}/h x {:03} = {}",
                payment_type.light_combat(),
                self.payment.light_combat,
                bootstrap::format::money(
                    (self.payment.light_combat as f32 / 60f32 * payment_type.light_combat() as f32)
                        .ceil() as i32,
                    true
                )
            )
            .expect("should be able to write to string");
        }
        if self.payment.medium_combat > 0 {
            writeln!(
                math,
                "Medium     |  {:03}/h x {:03} = {}",
                payment_type.medium_combat(),
                self.payment.medium_combat,
                bootstrap::format::money(
                    (self.payment.medium_combat as f32 / 60f32
                        * payment_type.medium_combat() as f32)
                        .ceil() as i32,
                    true
                )
            )
            .expect("should be able to write to string");
        }
        if self.payment.heavy_combat > 0 {
            writeln!(
                math,
                "Heavy      |  {:03}/h x {:03} = {}",
                payment_type.heavy_combat(),
                self.payment.heavy_combat,
                bootstrap::format::money(
                    (self.payment.heavy_combat as f32 / 60f32 * payment_type.heavy_combat() as f32)
                        .ceil() as i32,
                    true
                )
            )
            .expect("should be able to write to string");
        }
        writeln!(
            math,
            "Reputation | {}{:03.0}/h x {:03} = {}",
            if reputation < 0f32 { "-" } else { " " },
            reputation.abs().clamp(-300f32, 300f32) / 2f32,
            self.payment.total(),
            bootstrap::format::money(
                (reputation.clamp(-300f32, 300f32) / 120f32 * self.payment.total() as f32).ceil()
                    as i32,
                true
            )
        )
        .expect("should be able to write to string");
        writeln!(
            math,
            "{}    |  x{:.1}        = {}",
            self.outcome,
            self.outcome.contractor_multiplier(),
            bootstrap::format::money(self.contractor_payment(payment_type, reputation), true)
        )
        .expect("should be able to write to string");
        math.push('\n');
        writeln!(
            math,
            "Group      |  {:06}/h x {:03} = {}",
            payment_type.employer(),
            self.payment.total(),
            bootstrap::format::money(
                (self.payment.total() as f32 / 60f32 * payment_type.employer() as f32).ceil()
                    as i32,
                true
            )
        )
        .expect("should be able to write to string");
        writeln!(
            math,
            "Reputation | {}{:06}/h x {:03} = {}",
            if reputation < 0f32 { "-" } else { " " },
            reputation.abs() as i32 * 200 / 2,
            self.payment.total(),
            bootstrap::format::money(
                (self.payment.total() as f32 / 60f32 * reputation * 200f32 / 2f32).ceil() as i32,
                true
            )
        )
        .expect("should be able to write to string");
        writeln!(
            math,
            "{}    |  x{:.1}           = {}",
            self.outcome,
            self.outcome.employer_multiplier(),
            bootstrap::format::money(self.employer_payment(payment_type, reputation), true)
        )
        .expect("should be able to write to string");
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

#[allow(clippy::struct_field_names)]
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
    /// Returns the total time for the mission.
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
            // Self::ProtectionLowRisk => 420,
            // Self::ProtectionHighRisk => 490,
            // Self::Logistics => 280,
            // Self::Recon => 280,
            // Self::Offensive => 350,
            // Self::Defensive => 420,
            // Self::Support => 280,
            // Self::Security => 350,
            // Self::SmashGrab => 420,
            // bump 15%
            Self::ProtectionLowRisk => 483,
            Self::ProtectionHighRisk => 563,
            Self::Logistics => 322,
            Self::Recon => 322,
            Self::Offensive => 402,
            Self::Defensive => 483,
            Self::Support => 322,
            Self::Security => 402,
            Self::SmashGrab => 483,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for light combat.
    pub const fn light_combat(&self) -> i32 {
        match self {
            // Self::ProtectionLowRisk => 490,
            // Self::ProtectionHighRisk => 560,
            // Self::Logistics => 350,
            // Self::Recon => 350,
            // Self::Offensive => 420,
            // Self::Defensive => 490,
            // Self::Support => 350,
            // Self::Security => 420,
            // Self::SmashGrab => 490,
            // bump 15%
            Self::ProtectionLowRisk => 564,
            Self::ProtectionHighRisk => 644,
            Self::Logistics => 402,
            Self::Recon => 402,
            Self::Offensive => 483,
            Self::Defensive => 564,
            Self::Support => 402,
            Self::Security => 483,
            Self::SmashGrab => 564,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for medium combat.
    pub const fn medium_combat(&self) -> i32 {
        match self {
            // Self::ProtectionLowRisk => 560,
            // Self::ProtectionHighRisk => 700,
            // Self::Logistics => 420,
            // Self::Recon => 420,
            // Self::Offensive => 490,
            // Self::Defensive => 560,
            // Self::Support => 420,
            // Self::Security => 490,
            // Self::SmashGrab => 350,
            // bump 15%
            Self::ProtectionLowRisk => 644,
            Self::ProtectionHighRisk => 805,
            Self::Logistics => 483,
            Self::Recon => 483,
            Self::Offensive => 564,
            Self::Defensive => 644,
            Self::Support => 483,
            Self::Security => 564,
            Self::SmashGrab => 402,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for heavy combat.
    pub const fn heavy_combat(&self) -> i32 {
        match self {
            // Self::ProtectionLowRisk => 700,
            // Self::ProtectionHighRisk => 840,
            // Self::Logistics => 560,
            // Self::Recon => 560,
            // Self::Offensive => 560,
            // Self::Defensive => 630,
            // Self::Support => 560,
            // Self::Security => 560,
            // Self::SmashGrab => 210,
            // bump 15%
            Self::ProtectionLowRisk => 805,
            Self::ProtectionHighRisk => 966,
            Self::Logistics => 644,
            Self::Recon => 644,
            Self::Offensive => 644,
            Self::Defensive => 724,
            Self::Support => 644,
            Self::Security => 644,
            Self::SmashGrab => 241,
        }
    }

    #[must_use]
    #[allow(clippy::match_same_arms)]
    /// Hourly rate for the employer
    pub const fn employer(&self) -> i32 {
        (match self {
            // Self::ProtectionLowRisk => 59800,
            // Self::ProtectionHighRisk => 89700,
            // Self::Logistics => 44850,
            // Self::Recon => 44850,
            // Self::Offensive => 59800,
            // Self::Defensive => 67275,
            // Self::Support => 44850,
            // Self::Security => 59800,
            // Self::SmashGrab => 44850,
            // bump 25%
            Self::ProtectionLowRisk => 74750,
            Self::ProtectionHighRisk => 112_125,
            Self::Logistics => 56063,
            Self::Recon => 56063,
            Self::Offensive => 74750,
            Self::Defensive => 84094,
            Self::Support => 56063,
            Self::Security => 74750,
            Self::SmashGrab => 56063,
        }) * 2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_aar() {
        let aar = Aar::from_message(
            r"```Contract: Pusherman
Date: 2022-12-17
OL: Jake King
ELs: Carson Sering (KIA), Chaplain Yi (After Casualty), John Lamb (KIA)

Contractors: Jake King, Brett Harrison, Nathanial Greene, Carson Sering, Sean Miles, Chaplain Yi, Matías Jackson, John Brown, John Lamb
Assets Deployed: 2x Arcadian
Assets Lost: None
Casualties: John Lamb, Carson Sering

AAR: Contractors were tasked with destroying Cartel Assets on the island nation of Tanoa. Contractors began by assaulting the airfield. There was a handful of armed cartel members present at the airfield, which were taken out in the assault. A Cartel owned C-130, Mohawk Transport Helicopter, and Fuel Truck was destroyed. The ATC tower was resistant to demolition charges, so one was set in the tower itself to destroy the electronics inside of the tower itself. From the airfield, contractors regrouped at the FOB, and then began to make their way to the town of Tavu. A siege of the town began to neutralize the cartel presence within the town. In the early moments of contact with the cartel, both FTL's were KIA. After regrouping and triaging patients, the remaining cartel members were neutralized. 7 cartel owned RHIB's were destroyed, 2 cartel owned vans were destroyed, and 5 crates of cartel product were destroyed. Contractors successfully exfil'd back to the fob.

Operation Successful.

Payment request: 60 No Combat 30 Light Combat 45 Medium Combat 15 Heavy Combat```",
        );
        assert!(aar.is_ok());
        let aar = aar.expect("aar should be ok");
        println!("0");
        println!("{}", aar.show_math(PaymentType::Defensive, 0f32));
        println!("1");
        println!("{}", aar.show_math(PaymentType::Defensive, 1f32));
        println!("20");
        println!("{}", aar.show_math(PaymentType::Defensive, 20f32));
        println!("200");
        println!("{}", aar.show_math(PaymentType::Defensive, 200f32));
        println!("-1");
        println!("{}", aar.show_math(PaymentType::Defensive, -1f32));
        println!("-20");
        println!("{}", aar.show_math(PaymentType::Defensive, -20f32));
        println!("-200");
        println!("{}", aar.show_math(PaymentType::Defensive, -200f32));
    }

    #[test]
    fn test_parse_aar2() {
        let aar = Aar::from_message(
            r"```Contract: Safety
Date: 2023-03-25
OL: Thomas Watson
ELs: Jake King

Contractors: Thomas Watson, Gary McLean, Brett Harrison, Prince Singh, Sean Miles, Arsey Johnson, Brett Harrison, Jake King, Felix de Jong, Emerson Thoreau, John Lamb, Chaplain Yi, Andrew Libby
Assets Deployed: 2x Land Rover
Assets Lost: N/A
Casualties: Thomas Watson, Chaplain Yi, Brett Harrison, Prince Singh, Arsey Johnson, Gary McLean

AAR: Contractors split into two teams between Mandalaria and Barawas (Yellow and Blue, respectively). At Barawas, Blue team took heavy contact, requiring assistance from a mobilized Yellow team (that at the time was checking Kouble Maimatara). After the end of the firefight, Yellow moved to Mimi, to check the area for any ISAS activity, before returning to to Mandalari. At 0800, Blue was experiencing no contact, while Yellow was awaiting the arrival of the convoy to Mandalari. The convoy was taken out by RPG fire, and contractors engaged the sparse contact that remained after the main force retreated. Afterwards, Yellow sent a two man team to patrol Kouble Maimatara, ending up surrounding and KIA. The rest of Synixe contractors assaulted the town to upend the ISAS fighters that were in the town, and recover the bodies of the 2 man team. After 20 or so minutes of heavy combat, where contractors were assaulted from all directions, ISAS retreated from the town, only after the number of KIA contractors tripled (All but one of Yellow was KIA, One of Blue was KIA). Bodies were secured, and the remainder of the contractors RTB'd.

Operation Partial Success

Payment Request
50 No Combat
10 Light Combat
10 Medium Combat
30 Heavy Combat
```",
        );
        assert!(aar.is_err()); // Duplicate Brett Harrison
    }

    #[test]
    fn test_parse_color() {
        let aar = Aar::from_message(
            r"```ansi
Contract: Safety
Date: 2023-03-25
OL: Thomas Watson
ELs: Jake King

Contractors: [2;33mBrett Harrison[0m, [2;33mPrince Singh[0m, [2;31mTobias Jennings[0m, [2;31mChaplain Yi[0m, [2;31mArsey Johnson[0m, [2;31mAlex Drakulich[0m, [2;32mThomas Watson[0m, [2;32mGary McLean[0m, [2;32mAnthony Collins[0m, [2;32mVerdel Ricksin[0m, [2;34mAndrew Munson[0m, [2;34mChris Gibson[0m, [2;34mNorman Lennox[0m, [2;34mLeroy Nimrod[0m
Assets Deployed: 2x Land Rover
Assets Lost: N/A
Casualties: Thomas Watson, Chaplain Yi, Brett Harrison, Prince Singh, Arsey Johnson, Gary McLean

AAR: Contractors split into two teams between Mandalaria and Barawas (Yellow and Blue, respectively). At Barawas, Blue team took heavy contact, requiring assistance from a mobilized Yellow team (that at the time was checking Kouble Maimatara). After the end of the firefight, Yellow moved to Mimi, to check the area for any ISAS activity, before returning to to Mandalari. At 0800, Blue was experiencing no contact, while Yellow was awaiting the arrival of the convoy to Mandalari. The convoy was taken out by RPG fire, and contractors engaged the sparse contact that remained after the main force retreated. Afterwards, Yellow sent a two man team to patrol Kouble Maimatara, ending up surrounding and KIA. The rest of Synixe contractors assaulted the town to upend the ISAS fighters that were in the town, and recover the bodies of the 2 man team. After 20 or so minutes of heavy combat, where contractors were assaulted from all directions, ISAS retreated from the town, only after the number of KIA contractors tripled (All but one of Yellow was KIA, One of Blue was KIA). Bodies were secured, and the remainder of the contractors RTB'd.

Operation Partial Success

Payment Request
50 No Combat
10 Light Combat
10 Medium Combat
30 Heavy Combat
```",
        );
        assert!(aar.is_ok());
    }
}
