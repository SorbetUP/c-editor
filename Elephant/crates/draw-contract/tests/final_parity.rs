use elephant_draw_contract::{
    DrawingScene, ImageCrop, LibraryFile, LibraryItemStatus, SearchField, SelectionSet, SnapAxis,
    MINIMAL_CROP_SIZE,
};
use serde_json::json;

#[test]
fn transform_snap_crop_search_and_library_contracts_match_excalidraw_shapes() {
    let mut scene = DrawingScene::from_json(
        &serde_json::to_string(&json!({
            "type":"excalidraw",
            "elements":[
                {"id":"frame","type":"frame","x":0,"y":0,"width":200,"height":120,"name":"Roadmap Alpha","version":1},
                {"id":"child","type":"rectangle","x":20,"y":20,"width":40,"height":30,"frameId":"frame","boundElements":[{"type":"text","id":"text"}],"version":1},
                {"id":"text","type":"text","x":25,"y":25,"width":30,"height":20,"text":"Database cluster","containerId":"child","frameId":"frame","fontSize":20,"version":1},
                {"id":"line","type":"line","x":80,"y":20,"width":40,"height":20,"points":[[0,0],[40,20]],"frameId":"frame","version":1},
                {"id":"image","type":"image","x":220,"y":20,"width":100,"height":80,"crop":null,"version":1},
                {"id":"target","type":"rectangle","x":400,"y":100,"width":60,"height":60,"link":"https://example.com/docs"}
            ],"appState":{},"files":{}
        }))
        .unwrap(),
    )
    .unwrap();

    let frame = SelectionSet::from_ids(["frame".to_owned()]);
    let child_before = scene.element_by_id("child").unwrap().clone();
    let line_before = scene.element_by_id("line").unwrap().clone();
    assert_eq!(scene.scale_selection(&frame, [0.0, 0.0], [2.0, 2.0]).changed, 1);
    assert_eq!(scene.element_by_id("frame").map(|e| (e.width, e.height)), Some((400.0, 240.0)));
    assert_eq!(scene.element_by_id("child").unwrap(), &child_before);
    assert_eq!(scene.element_by_id("line").unwrap(), &line_before);
    assert_eq!(scene.rotate_selection(&frame, std::f32::consts::FRAC_PI_2).changed, 0);
    assert!(scene.rotate_selection(&SelectionSet::from_ids(["child".to_owned()]), std::f32::consts::FRAC_PI_2).changed > 0);

    let moving = SelectionSet::from_ids(["image".to_owned()]);
    let snap = scene.snap_selection_delta(&moving, [77.0, 0.0], 5.0);
    assert!(snap.guides.iter().all(|guide| matches!(guide.axis, SnapAxis::X | SnapAxis::Y)));

    assert!(scene.set_image_crop("image", Some(ImageCrop {
        x: 90.0,
        y: 70.0,
        width: 1.0,
        height: 2.0,
        natural_width: 120.0,
        natural_height: 90.0,
    })));
    let crop = scene.image_crop("image").unwrap();
    assert_eq!((crop.width, crop.height), (MINIMAL_CROP_SIZE, MINIMAL_CROP_SIZE));
    assert_eq!(scene.element_by_id("image").unwrap().extra["crop"]["naturalWidth"], 120.0);

    assert_eq!(scene.search_scene("alpha")[0].field, SearchField::FrameName);
    assert_eq!(scene.search_scene("database")[0].field, SearchField::Text);
    assert_eq!(scene.search_scene("example.com")[0].field, SearchField::Link);
    assert!(scene.set_element_link("child", Some("openai.com")));
    assert_eq!(scene.element_link("child"), Some("https://openai.com"));

    let mut library = LibraryFile::empty("elephant");
    assert!(library.add_from_selection(&scene, &frame, "item-1", 42, Some("Architecture".into())));
    assert_eq!(library.file_type, "excalidrawlib");
    assert_eq!(library.version, 2);
    assert_eq!(library.library_items[0].status, LibraryItemStatus::Unpublished);
    assert_eq!(library.search("database").len(), 1);
    let raw = serde_json::to_string(&library).unwrap();
    assert_eq!(serde_json::from_str::<LibraryFile>(&raw).unwrap(), library);

    let mut target = DrawingScene::empty();
    let inserted = target.insert_library_item(&library.library_items[0], [10.0, 10.0], "lib1");
    assert!(!inserted.is_empty());
    assert_eq!(target.element_by_id("lib1-text").unwrap().extra["containerId"], "lib1-child");
}
