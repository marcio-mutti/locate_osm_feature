use core::panic;
use std::{
    collections::HashMap,
    io::{BufReader, Seek},
    path::Path,
};

use geo::{Centroid, Coord, LineString, Polygon};
use indicatif::{ProgressBar, ProgressStyle};
use quick_xml::{events::Event, Reader};

use crate::tipos::{AreaOSM, FeatureOSM, NoOSM};

pub fn carregar_osm(caminho: impl AsRef<Path>) -> Result<Vec<NoOSM>, Box<dyn std::error::Error>> {
    let mut tamanho: u64 = Default::default();

    let estilo_barra = ProgressStyle::with_template(
        "{prefix} [{elapsed_precise}] {bar:40.cyan/blue} [{eta_precise}] [{msg}]",
    )?
    .progress_chars("█▇▆▅▄▃▂▁  ");
    let mut n_tags: usize = Default::default();
    let mut buffer_xml = Vec::new();
    {
        let mut arq_osm = std::fs::File::open(caminho.as_ref())?;
        arq_osm.seek(std::io::SeekFrom::End(0))?;
        tamanho = arq_osm.stream_position()?;
        arq_osm.seek(std::io::SeekFrom::Start(0))?;
        let arq_osm = BufReader::new(arq_osm);
        let mut arq_xml = Reader::from_reader(arq_osm);
        let leitor = arq_xml.expand_empty_elements(true);
        let progresso_leitura_inicial = ProgressBar::new(tamanho)
            .with_style(estilo_barra.clone())
            .with_prefix("Calcaulando o tamanho total.");
        loop {
            progresso_leitura_inicial.set_position(leitor.buffer_position() as u64);
            match leitor.read_event_into(&mut buffer_xml) {
                Ok(evento) => match evento {
                    Event::Start(elemento) => match elemento.as_ref() {
                        b"tag" => {
                            n_tags += 1;
                        }
                        _ => {}
                    },
                    Event::Eof => break,
                    _ => {}
                },
                Err(e) => panic!(
                    "Erro ao lero arquivo OSM na posicao {}.\n{:#?}",
                    leitor.buffer_position(),
                    e
                ),
            }
        }
        progresso_leitura_inicial.finish_with_message("Leitura Inicial Completa.");
    }
    let arq_osm = std::fs::File::open(caminho.as_ref())?;
    let arq_osm = BufReader::new(arq_osm);
    let progresso = indicatif::ProgressBar::new(tamanho)
        .with_style(estilo_barra.clone())
        .with_prefix("Carga OSM");
    //let mut arq_osm = std::fs::File::open(caminho.as_ref())?;
    let mut arq_xml = Reader::from_reader(arq_osm);
    let mut buffer_xml = Vec::new();
    let mut lista_nos_medicina: Vec<NoOSM> = Vec::new();
    let mut lista_de_areas: Vec<AreaOSM> = Vec::new();
    let mut nos_coordenadas: HashMap<i64, Coord<f64>> = HashMap::with_capacity(n_tags);
    {
        let leitor = arq_xml.expand_empty_elements(true);
        let mut elemento_atual: FeatureOSM = Default::default();
        loop {
            let posicao_atual = leitor.buffer_position();
            match leitor.read_event_into(&mut buffer_xml) {
                Ok(evento) => match evento {
                    Event::Start(elemento) => match elemento.name().as_ref() {
                        b"node" => {
                            let mut id: i64 = Default::default();
                            let mut latitude: f64 = Default::default();
                            let mut longitude: f64 = Default::default();
                            for attr in elemento.attributes() {
                                if let Ok(attr) = attr {
                                    match attr.key.as_ref() {
                                        b"id" => {
                                            if let Ok(n_id) =
                                                String::from_utf8_lossy(&attr.value).parse::<i64>()
                                            {
                                                id = n_id;
                                            }
                                        }
                                        b"lat" => {
                                            if let Ok(lat) =
                                                String::from_utf8_lossy(&attr.value).parse::<f64>()
                                            {
                                                latitude = lat;
                                            }
                                        }
                                        b"lon" => {
                                            if let Ok(long) =
                                                String::from_utf8_lossy(&attr.value).parse::<f64>()
                                            {
                                                longitude = long;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            elemento_atual =
                                FeatureOSM::No(NoOSM::novo(id, None, latitude, longitude, None));
                        }
                        b"way" => {
                            let mut id: i64 = Default::default();
                            for attr in elemento.attributes() {
                                if let Ok(attr) = attr {
                                    if attr.key.as_ref() == b"id" {
                                        if let Ok(n_id) =
                                            String::from_utf8_lossy(&attr.value).parse::<i64>()
                                        {
                                            id = n_id;
                                            break;
                                        }
                                    }
                                }
                            }
                            elemento_atual = FeatureOSM::Area(AreaOSM::novo(id, None, None));
                        }
                        b"tag" => {
                            let mut chave: String = String::new();
                            let mut valor: String = String::new();
                            for attr in elemento.attributes() {
                                if let Ok(attr) = attr {
                                    match attr.key.as_ref() {
                                        b"k" => {
                                            chave = String::from_utf8_lossy(&attr.value).to_string()
                                        }
                                        b"v" => {
                                            valor = String::from_utf8_lossy(&attr.value).to_string()
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            elemento_atual.add_tag(chave, valor);
                        }
                        b"nd" => {
                            for attr in elemento.attributes() {
                                if let Ok(attr) = attr {
                                    if attr.key.as_ref() == b"ref" {
                                        if let Ok(id) =
                                            String::from_utf8_lossy(&attr.value).parse::<i64>()
                                        {
                                            if let FeatureOSM::Area(ref mut area) = elemento_atual {
                                                area.adicionar_no_coordenada(id);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Event::End(elemento) => {
                        match elemento.name().as_ref() {
                            b"node" | b"way" => match elemento_atual.e_medicina() {
                                true => match elemento_atual {
                                    FeatureOSM::No(no) => lista_nos_medicina.push(no),
                                    FeatureOSM::Area(area) => lista_de_areas.push(area),
                                    FeatureOSM::Undefined => {}
                                },
                                false => {
                                    if let FeatureOSM::No(ref no) = elemento_atual {
                                        nos_coordenadas.insert(no.id(), no.coordenada());
                                    }
                                }
                            },
                            _ => {}
                        }

                        elemento_atual = Default::default();
                    }
                    Event::Eof => break,
                    _ => {}
                },
                Err(e) => panic!(
                    "Erro ao ler o arquivo OSM no posição {}.\n{:?}",
                    posicao_atual, e
                ),
            }
            progresso.set_position(posicao_atual as u64);
        }
    }
    progresso.finish_with_message("Carga encerrada");
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
                    area.nome().map(|s| s.clone()),
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
