use locate_osm_feature::funcoes::carregar_osm;

#[test]
fn teste_0() {
    
}

#[test]
fn testar_extrato() {
    let carga = carregar_osm("/home/marcio/Documentos/Rust/Pessoal/OSM/extrato.osm").unwrap();
    println!("Carregados {} registros.", carga.len());
}

#[test]
fn testar_nordeste() {
    let carga = carregar_osm("/home/marcio/Documentos/Rust/Pessoal/OSM/nordeste-latest.osm").unwrap();
    println!("Carregados {} registros.", carga.len());

}