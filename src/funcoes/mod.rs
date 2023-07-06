use core::panic;
use std::{
    collections::HashMap,
    io::BufReader,
    path::Path,
};

use geo::Coord;
use indicatif::{ProgressBar, ProgressStyle};
use xml::reader::XmlEvent;
use xml::ParserConfig;

use crate::tipos::{AreaOSM, FeatureOSM, NoOSM};

pub fn carregar_osm(caminho: impl AsRef<Path>) -> Result<Vec<NoOSM>, Box<dyn std::error::Error>> {
    let estilo_barra = ProgressStyle::with_template(
        "{prefix} [{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] [{msg}]",
    )?
    .progress_chars("█▇▆▅▄▃▂▁  ");
    let arq_osm = std::fs::File::open(caminho.as_ref())?;
    let arq_osm = BufReader::new(arq_osm);
    let parser_xml = ParserConfig::new()
        .trim_whitespace(true)
        .ignore_comments(true)
        .create_reader(arq_osm);
    let mut lista_nos_medicina: Vec<NoOSM> = Vec::new();
    let mut lista_de_areas: Vec<AreaOSM> = Vec::new();
    let mut nos_coordenadas: HashMap<i64, Coord<f64>> = HashMap::new();
    let mut elemento_atual: FeatureOSM = Default::default();
    for elemento in parser_xml {
        match elemento {
            Ok(elemento) => match elemento {
                XmlEvent::StartElement {
                    name, attributes, ..
                } => match name.local_name.as_str() {
                    "node" => {
                        let mut id: i64 = Default::default();
                        let mut latitude: f64 = Default::default();
                        let mut longitude: f64 = Default::default();
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "id" => {
                                    if let Ok(n_id) = attr.value.parse() {
                                        id = n_id;
                                    }
                                }
                                "lat" => {
                                    if let Ok(n_lat) = attr.value.parse() {
                                        latitude = n_lat;
                                    }
                                }
                                "lon" => {
                                    if let Ok(n_lon) = attr.value.parse() {
                                        longitude = n_lon;
                                    }
                                }
                                _ => {}
                            }
                        }
                        elemento_atual =
                            FeatureOSM::No(NoOSM::novo(id, None, latitude, longitude, None));
                    }
                    "way" => {
                        let mut id: i64 = Default::default();
                        if let Some(attr) = attributes
                            .into_iter()
                            .find(|a| a.name.local_name.as_str() == "id")
                        {
                            if let Ok(n_id) = attr.value.parse() {
                                id = n_id;
                            }
                        }
                        elemento_atual = FeatureOSM::Area(AreaOSM::novo(id, None, None));
                    }
                    "tag" => {
                        let mut chave: String = String::new();
                        let mut valor: String = String::new();
                        for attr in attributes {
                            match attr.name.local_name.as_str() {
                                "k" => chave = attr.value,
                                "v" => valor = attr.value,
                                _ => {}
                            }
                        }
                        elemento_atual.add_tag(chave, valor);
                    }
                    "nd" => {
                        attributes
                            .into_iter()
                            .filter(|a| a.name.local_name.as_str() == "ref")
                            .for_each(|a| {
                                if let Ok(id) = a.value.parse() {
                                    if let FeatureOSM::Area(ref mut area) = elemento_atual {
                                        area.adicionar_no_coordenada(id);
                                    }
                                }
                            });
                    }
                    _ => {}
                },
                XmlEvent::EndElement { name } => {
                    match name.local_name.as_str() {
                        "node" | "way" => {
                            match elemento_atual.e_medicina() {
                                true => match elemento_atual {
                                    FeatureOSM::No(no) => lista_nos_medicina.push(no),
                                    FeatureOSM::Area(area) => lista_de_areas.push(area),
                                    FeatureOSM::Indefinido => {}
                                },
                                false => if let FeatureOSM::No(ref no) = elemento_atual {
                                        nos_coordenadas.insert(no.id(), no.coordenada());
                                    },
                            }
                            elemento_atual = Default::default();
                        }
                        _ => {}
                    }
                    
                },
                _ => {}
            },
            Err(erro) => panic!("Erro encontrado ao ler arquivo OSM.\n{:#?}", erro),
        }
    }
    lista_nos_medicina.reserve_exact(lista_de_areas.len());
    let progresso_transformacao = ProgressBar::new(lista_de_areas.len() as u64)
        .with_style(estilo_barra)
        .with_prefix("Transformar as áreas");
    for area in lista_de_areas {
        progresso_transformacao.inc(1);
        if let Some(idf) = area
            .ref_nos()
            .iter()
            .find(|&i| nos_coordenadas.contains_key(i))
        {
            if let Some(c) = nos_coordenadas.get(idf) {
                lista_nos_medicina.push(NoOSM::novo(
                    *area.id(),
                    area.nome().cloned(),
                    c.y,
                    c.x,
                    Some(true),
                ));
            }
        }
        //if let Some(c) = nos_coordenadas.iter().find_map(|(i,nc)|)
        /* let poligono_area = Polygon::new(
            LineString::from_iter(
                area.ref_nos()
                    .iter()
                    .map(|n_id| nos_coordenadas.get(n_id).unwrap())
                    .copied(),
            ),
            Vec::new(),
        );
        let centroide = poligono_area.centroid().unwrap();
        let no = NoOSM::novo(
            *area.id(),
            area.nome().map(|s| s.clone()),
            centroide.y(),
            centroide.x(),
            Some(area.e_medicina()),
        );*/
        //lista_nos_medicina.push(no);
    }
    progresso_transformacao.finish_with_message("Transformações encerradas");
    Ok(lista_nos_medicina)
}
