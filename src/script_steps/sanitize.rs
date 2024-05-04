use crate::script_steps::constants::{id_to_script_step, ScriptStep};
use quick_xml::events::Event;
use quick_xml::Reader;

use crate::script_steps::parameters::parameter_values::ParameterValues;
use crate::utils::attributes::get_attribute;

pub fn from_xml(step_id: &u32, step: &str) -> Option<String> {
    let mut name = String::new();
    let mut parameters: Vec<String> = Vec::new();

    let mut reader = Reader::from_str(step);
    let mut buf: Vec<u8> = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Err(_) => continue,
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => match e.name().as_ref() {
                b"Step" => {
                    if let Some(value) = get_attribute(&e, "name") {
                        name = value.to_string();
                    }
                    continue;
                }
                b"ParameterValues" => parameters.push(
                    ParameterValues::from_xml(&mut reader, &e, step_id)
                        .unwrap()
                        .display()
                        .unwrap(),
                ),
                _ => {}
            },
            _ => {}
        }
        buf.clear()
    }

    let parameters = parameters.join(" ; ");

    if id_to_script_step(step_id) == ScriptStep::Comment {
        if parameters.trim().is_empty() {
            return Some("".to_string());
        } else {
            return Some(format!("# {}", parameters));
        }
    }

    let parameters = parameters.trim();
    if parameters.is_empty() {
        return Some(name.to_string());
    };

    Some(format!("{} [ {} ]", name, parameters))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"<Step id="79" name="Fenster fixieren" enable="True">"#;

        let expected_output = Some("Fenster fixieren".to_string());
        let script_id: u32 = 79;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_enabled() {
        let xml = r#"
            <Step id="86" name="Fehleraufzeichnung setzen" enable="True">
                <Options>196608</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean id="131072" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Fehleraufzeichnung setzen [ ON ]".to_string());
        let script_id: u32 = 86;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_disabled() {
        let xml = r#"
            <Step id="86" name="Fehleraufzeichnung setzen" enable="True">
                <Options>196608</Options>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean id="131072" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Fehleraufzeichnung setzen [ OFF ]".to_string());
        let script_id: u32 = 86;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_enter_find_mode() {
        let xml = r#"
            <Step id="22" name="Suchenmodus aktivieren" enable="True">
                <SourceUUID>3C5EBF9C-BAE0-460F-92AF-3FB73649951B</SourceUUID>
                <ParameterValues membercount="1">
                    <Parameter type="Boolean">
                        <Boolean type="Pause" id="16777216" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Suchenmodus aktivieren [ Pause: OFF ]".to_string());
        let script_id: u32 = 22;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_truncate_table_broken_reference() {
        let xml = r#"
            <Step index="509" id="182" name="Tabelle leeren" enable="True">
                <UUID>25ED2DB6-D1D0-402A-8B5A-6A06505C0A2A</UUID>
                <OwnerID></OwnerID>
                <Options>138</Options>
                <ParameterValues membercount="2">
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="List">
                        <List name="&lt;Tabelle nicht vorhanden&gt;" value="1"></List>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Tabelle leeren [ Mit Dialog: OFF ; <Tabelle nicht vorhanden> ]".to_string());
        let script_id: u32 = 182;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }
}

#[cfg(test)]
mod commit_tests {
    use super::*;

    #[test]
    fn test() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output =
            Some("Schreibe Änderung Datens./Abfrage [ Mit Dialog: OFF ]".to_string());
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_force() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some(
            "Schreibe Änderung Datens./Abfrage [ Mit Dialog: OFF ; Schreiben erzwingen ]"
                .to_string(),
        );
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_suppress_validate() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="False"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="False"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Schreibe Änderung Datens./Abfrage [ Dateneingabeüberprüfung unterdrücken ; Mit Dialog: OFF ]".to_string());
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }

    #[test]
    fn test_all_options() {
        let xml = r#"
            <Step id="75" name="Schreibe Änderung Datens./Abfrage" enable="True">
                <Options>384</Options>
                <ParameterValues membercount="3">
                    <Parameter type="Boolean">
                        <Boolean type="Dateneingabeüberprüfung unterdrücken" id="256" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Mit Dialog" id="128" value="True"></Boolean>
                    </Parameter>
                    <Parameter type="Boolean">
                        <Boolean type="Schreiben erzwingen" id="512" value="True"></Boolean>
                    </Parameter>
                </ParameterValues>
            </Step>
        "#;

        let expected_output = Some("Schreibe Änderung Datens./Abfrage [ Dateneingabeüberprüfung unterdrücken ; Mit Dialog: ON ; Schreiben erzwingen ]".to_string());
        let script_id: u32 = 75;
        assert_eq!(from_xml(&script_id, xml.trim()), expected_output);
    }
}
