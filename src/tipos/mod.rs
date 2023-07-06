use std::fmt::Display;

use geo::{Coord, Point};
use wkt::ToWkt;

#[derive(Default, Debug)]
pub enum FeatureOSM {
    No(NoOSM),
    Area(AreaOSM),
    #[default]
    Indefinido,
}

impl FeatureOSM {
    pub fn e_medicina(&self) -> bool {
        match self {
            FeatureOSM::No(ref no) => no.e_medicina(),
            FeatureOSM::Area(ref area) => area.e_medicina(),
            FeatureOSM::Indefinido => false,
        }
    }
    pub fn add_tag(&mut self, chave: String, valor: String) {
        if chave.as_str() == "name" {
            match self {
                FeatureOSM::No(no) => no.informar_nome(valor),
                FeatureOSM::Area(area) => area.informar_nome(valor),
                FeatureOSM::Indefinido => {}
            }
        } else {
            let setar_medicina = match chave.as_str() {
                "healthcare" => true,
                "amenity" => matches!(valor.as_str(),"clinic" | "doctors" | "nursing_home" | "hospital" | "pharmacy" | "social_facility"),
                _ => false,
            };
            if setar_medicina {
                match self {
                    FeatureOSM::No(node) => node.confirmar_medicina(),
                    FeatureOSM::Area(area) => area.confirmar_medicina(),
                    FeatureOSM::Indefinido => {}
                }
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct NoOSM {
    coordenada: Point<f64>,
    id: i64,
    name: Option<String>,
    e_medicina: bool,
}

impl NoOSM {
    pub fn novo(
        id: i64,
        name: Option<String>,
        latitude: f64,
        longitude: f64,
        e_medicina: Option<bool>,
    ) -> Self {
        let coordenada = Point::new(longitude, latitude);
        Self {
            coordenada,
            id,
            name,
            e_medicina: e_medicina.unwrap_or(false),
        }
    }
    pub fn confirmar_medicina(&mut self) {
        self.e_medicina = true;
    }
    pub fn e_medicina(&self) -> bool {
        self.e_medicina
    }
    pub fn id(&self) -> i64 {
        self.id
    }
    pub fn coordenada(&self) -> Coord<f64> {
        Coord {
            x: self.coordenada.x(),
            y: self.coordenada.y(),
        }
    }
    pub fn informar_nome(&mut self, nome: String) {
        self.name = Some(nome);
    }
    pub fn titulo_csv() -> &'static str {
        "Nome;Ponto_WKT;ID_OSM"
    }
}
impl Display for NoOSM {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{};{};{}",
            self.name.as_ref().unwrap_or(&String::from("")),
            self.coordenada.to_wkt(),
            self.id
        )
    }
}
#[derive(Debug, Default, Clone)]
pub struct AreaOSM {
    lista_nos: Vec<i64>,
    id: i64,
    name: Option<String>,
    e_medicina: bool,
}

impl AreaOSM {
    pub fn novo(id: i64, name: Option<String>, e_medicina: Option<bool>) -> Self {
        Self {
            lista_nos: Vec::new(),
            id,
            name,
            e_medicina: e_medicina.unwrap_or(false),
        }
    }
    pub fn confirmar_medicina(&mut self) {
        self.e_medicina = true;
    }
    pub fn e_medicina(&self) -> bool {
        self.e_medicina
    }
    pub fn adicionar_no_coordenada(&mut self, no: i64) {
        self.lista_nos.push(no);
    }
    pub fn informar_nome(&mut self, nome: String) {
        self.name = Some(nome);
    }
    pub fn ref_nos(&self) -> &[i64] {
        &self.lista_nos
    }
    pub fn nome(&self) -> Option<&String> {
        self.name.as_ref()
    }
    pub fn id(&self) -> &i64 {
        &self.id
    }
}
