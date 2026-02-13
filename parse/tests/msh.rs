//! MSH parser tests.

use parse::msh;

#[test]
fn msh_nodes_and_tets() {
    let data = b"$MeshFormat
2.2 0 8
$EndMeshFormat
$Nodes
4
1 0 0 0
2 1 0 0
3 0 1 0
4 0 0 1
$EndNodes
$Elements
1
1 4 0 1 2 3 4
$EndElements";
    let msh = msh::parse(data).unwrap();
    assert_eq!(msh.positions.len(), 4);
    assert_eq!(msh.tets.len(), 1);
    assert_eq!(msh.tets[0], [0, 1, 2, 3]);
}
