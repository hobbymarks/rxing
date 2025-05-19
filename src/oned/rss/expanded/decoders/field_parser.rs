/*
 * Copyright (C) 2010 ZXing authors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

/*
 * These authors would like to acknowledge the Spanish Ministry of Industry,
 * Tourism and Trade, for the support in the project TSI020301-2008-2
 * "PIRAmIDE: Personalizable Interactions with Resources on AmI-enabled
 * Mobile Dynamic Environments", led by Treelogic
 * ( http://www.treelogic.com/ ):
 *
 *   http://www.piramidepse.com/
 */
/**
 * @author Pablo Orduña, University of Deusto (pablo.orduna@deusto.es)
 * @author Eduardo Castillejo, University of Deusto (eduardo.castillejo@deusto.es)
 */
use std::collections::HashMap;

use crate::common::Result;
use crate::Exceptions;

use once_cell::sync::Lazy;

static TWO_DIGIT_DATA_LENGTH: Lazy<HashMap<String, DataLength>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert("00".into(), DataLength::fixed(18));
    hm.insert("01".into(), DataLength::fixed(14));
    hm.insert("02".into(), DataLength::fixed(14));
    hm.insert("10".into(), DataLength::variable(20));
    hm.insert("11".into(), DataLength::fixed(6));
    hm.insert("12".into(), DataLength::fixed(6));
    hm.insert("13".into(), DataLength::fixed(6));
    hm.insert("15".into(), DataLength::fixed(6));
    hm.insert("17".into(), DataLength::fixed(6));
    hm.insert("20".into(), DataLength::fixed(2));
    hm.insert("21".into(), DataLength::variable(20));
    hm.insert("22".into(), DataLength::variable(29));
    hm.insert("30".into(), DataLength::variable(8));
    hm.insert("37".into(), DataLength::variable(8));
    //internal company codes
    for i in 90..=99 {
        // for (int i = 90; i <= 99; i++) {
        hm.insert(i.to_string(), DataLength::variable(30));
    }
    hm
});

static THREE_DIGIT_DATA_LENGTH: Lazy<HashMap<String, DataLength>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert("240".into(), DataLength::variable(30));
    hm.insert("241".into(), DataLength::variable(30));
    hm.insert("242".into(), DataLength::variable(6));
    hm.insert("250".into(), DataLength::variable(30));
    hm.insert("251".into(), DataLength::variable(30));
    hm.insert("253".into(), DataLength::variable(17));
    hm.insert("254".into(), DataLength::variable(20));
    hm.insert("400".into(), DataLength::variable(30));
    hm.insert("401".into(), DataLength::variable(30));
    hm.insert("402".into(), DataLength::fixed(17));
    hm.insert("403".into(), DataLength::variable(30));
    hm.insert("410".into(), DataLength::fixed(13));
    hm.insert("411".into(), DataLength::fixed(13));
    hm.insert("412".into(), DataLength::fixed(13));
    hm.insert("413".into(), DataLength::fixed(13));
    hm.insert("414".into(), DataLength::fixed(13));
    hm.insert("420".into(), DataLength::variable(20));
    hm.insert("421".into(), DataLength::variable(15));
    hm.insert("422".into(), DataLength::fixed(3));
    hm.insert("423".into(), DataLength::variable(15));
    hm.insert("424".into(), DataLength::fixed(3));
    hm.insert("425".into(), DataLength::fixed(3));
    hm.insert("426".into(), DataLength::fixed(3));

    hm
});

static THREE_DIGIT_PLUS_DIGIT_DATA_LENGTH: Lazy<HashMap<String, DataLength>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    for i in 310..=316 {
        // for (int i = 310; i <= 316; i++) {
        hm.insert(i.to_string(), DataLength::fixed(6));
    }
    for i in 320..=336 {
        // for (int i = 320; i <= 336; i++) {
        hm.insert(i.to_string(), DataLength::fixed(6));
    }
    for i in 340..=357 {
        // for (int i = 340; i <= 357; i++) {
        hm.insert(i.to_string(), DataLength::fixed(6));
    }
    for i in 360..=369 {
        // for (int i = 360; i <= 369; i++) {
        hm.insert(i.to_string(), DataLength::fixed(6));
    }
    hm.insert("390".into(), DataLength::variable(15));
    hm.insert("391".into(), DataLength::variable(18));
    hm.insert("392".into(), DataLength::variable(15));
    hm.insert("393".into(), DataLength::variable(18));
    hm.insert("703".into(), DataLength::variable(30));

    hm
});

static FOUR_DIGIT_DATA_LENGTH: Lazy<HashMap<String, DataLength>> = Lazy::new(|| {
    let mut hm = HashMap::new();
    hm.insert("7001".into(), DataLength::fixed(13));
    hm.insert("7002".into(), DataLength::variable(30));
    hm.insert("7003".into(), DataLength::fixed(10));
    hm.insert("8001".into(), DataLength::fixed(14));
    hm.insert("8002".into(), DataLength::variable(20));
    hm.insert("8003".into(), DataLength::variable(30));
    hm.insert("8004".into(), DataLength::variable(30));
    hm.insert("8005".into(), DataLength::fixed(6));
    hm.insert("8006".into(), DataLength::fixed(18));
    hm.insert("8007".into(), DataLength::variable(30));
    hm.insert("8008".into(), DataLength::variable(12));
    hm.insert("8018".into(), DataLength::fixed(18));
    hm.insert("8020".into(), DataLength::variable(25));
    hm.insert("8100".into(), DataLength::fixed(6));
    hm.insert("8101".into(), DataLength::fixed(10));
    hm.insert("8102".into(), DataLength::fixed(2));
    hm.insert("8110".into(), DataLength::variable(70));
    hm.insert("8200".into(), DataLength::variable(70));

    hm
});

pub fn parseFieldsInGeneralPurpose(rawInformation: &str) -> Result<String> {
    if rawInformation.is_empty() {
        return Ok(String::default());
    }

    // Processing 2-digit AIs

    if rawInformation.chars().count() < 2 {
        return Err(Exceptions::NOT_FOUND);
    }

    let lookup: String = rawInformation.chars().take(2).collect();
    let twoDigitDataLength = TWO_DIGIT_DATA_LENGTH.get(&lookup);
    if let Some(tddl) = twoDigitDataLength {
        if tddl.variable {
            return processVariableAI(2, tddl.length, rawInformation);
        }
        return processFixedAI(2, tddl.length, rawInformation);
    }

    if rawInformation.chars().count() < 3 {
        return Err(Exceptions::NOT_FOUND);
    }

    let firstThreeDigits: String = rawInformation.chars().take(3).collect();
    let threeDigitDataLength = THREE_DIGIT_DATA_LENGTH.get(&firstThreeDigits);
    if let Some(tddl) = threeDigitDataLength {
        if tddl.variable {
            return processVariableAI(3, tddl.length, rawInformation);
        }
        return processFixedAI(3, tddl.length, rawInformation);
    }

    if rawInformation.chars().count() < 4 {
        return Err(Exceptions::NOT_FOUND);
    }

    let threeDigitPlusDigitDataLength = THREE_DIGIT_PLUS_DIGIT_DATA_LENGTH.get(&firstThreeDigits);
    if let Some(tdpddl) = threeDigitPlusDigitDataLength {
        if tdpddl.variable {
            return processVariableAI(4, tdpddl.length, rawInformation);
        }
        return processFixedAI(4, tdpddl.length, rawInformation);
    }

    let lookup: String = rawInformation.chars().take(4).collect();
    let firstFourDigitLength = FOUR_DIGIT_DATA_LENGTH.get(&lookup /*(0, 4)*/);
    if let Some(ffdl) = firstFourDigitLength {
        if ffdl.variable {
            return processVariableAI(4, ffdl.length, rawInformation);
        }
        return processFixedAI(4, ffdl.length, rawInformation);
    }

    Err(Exceptions::NOT_FOUND)
}

fn processFixedAI(aiSize: usize, fieldSize: usize, rawInformation: &str) -> Result<String> {
    if rawInformation.chars().count() < aiSize {
        return Err(Exceptions::NOT_FOUND);
    }

    let ai: String = rawInformation.chars().take(aiSize).collect();

    if rawInformation.chars().count() < aiSize + fieldSize {
        return Err(Exceptions::NOT_FOUND);
    }

    let field: String = rawInformation
        .chars()
        .skip(aiSize)
        .take(fieldSize)
        .collect(); //rawInformation.substring(aiSize, aiSize + fieldSize);
    let remaining: String = rawInformation.chars().skip(aiSize + fieldSize).collect(); // rawInformation.substring(aiSize + fieldSize);
    let result = format!("({ai}){field}");
    let parsedAI = parseFieldsInGeneralPurpose(&remaining)?;

    Ok(if parsedAI.is_empty() {
        result
    } else {
        format!("{result}{parsedAI}")
    })
}

fn processVariableAI(
    aiSize: usize,
    variableFieldSize: usize,
    rawInformation: &str,
) -> Result<String> {
    let ai: String = rawInformation.chars().take(aiSize).collect();
    let maxSize = rawInformation
        .chars()
        .count()
        .min(aiSize + variableFieldSize);
    let field: String = rawInformation.chars().skip(aiSize).take(maxSize).collect(); // (aiSize, maxSize);
    let remaining: String = rawInformation.chars().skip(maxSize).collect();
    let result = format!("({ai}){field}"); //'(' + ai + ')' + field;
    let parsedAI = parseFieldsInGeneralPurpose(&remaining)?;

    Ok(if parsedAI.is_empty() {
        result
    } else {
        format!("{result}{parsedAI}")
    })
}

struct DataLength {
    pub variable: bool,
    pub length: usize,
}
impl DataLength {
    // fn new( variable:bool, length:u32) -> Self{
    //   Self(variable,length)
    // }

    pub const fn fixed(length: usize) -> Self {
        Self {
            variable: false,
            length,
        }
    }

    pub const fn variable(length: usize) -> Self {
        Self {
            variable: true,
            length,
        }
    }
}

/**
 * @author Pablo Orduña, University of Deusto (pablo.orduna@deusto.es)
 * @author Eduardo Castillejo, University of Deusto (eduardo.castillejo@deusto.es)
 */
#[cfg(test)]
mod FieldParserTest {

    fn checkFields(expected: &str) {
        let field = expected.replace(['(', ')'], "");
        let actual = super::parseFieldsInGeneralPurpose(&field).expect("parse");
        assert_eq!(expected, actual);
    }

    #[test]
    fn testParseField() {
        checkFields("(15)991231(3103)001750(10)12A");
    }

    #[test]
    fn testParseField2() {
        checkFields("(15)991231(15)991231(3103)001750(10)12A");
    }
}
