use crate::grid::field_test::script::FieldScript::*;
use crate::grid::field_test::script::GridFieldTest;
use crate::grid::field_test::util::*;
use bytes::Bytes;
use flowy_grid::entities::{FieldChangesetParams, FieldType};
use flowy_grid::services::field::selection_type_option::SelectOptionPB;
use flowy_grid::services::field::{gen_option_id, SingleSelectTypeOptionPB, CHECK, UNCHECK};

#[tokio::test]
async fn grid_create_field() {
    let mut test = GridFieldTest::new().await;
    let (params, field_rev) = create_text_field(&test.grid_id());

    let scripts = vec![
        CreateField { params },
        AssertFieldTypeOptionEqual {
            field_index: test.field_count(),
            expected_type_option_data: field_rev.get_type_option_str(field_rev.ty).unwrap(),
        },
    ];
    test.run_scripts(scripts).await;

    let (params, field_rev) = create_single_select_field(&test.grid_id());
    let scripts = vec![
        CreateField { params },
        AssertFieldTypeOptionEqual {
            field_index: test.field_count(),
            expected_type_option_data: field_rev.get_type_option_str(field_rev.ty).unwrap(),
        },
    ];
    test.run_scripts(scripts).await;
}

#[tokio::test]
async fn grid_create_duplicate_field() {
    let mut test = GridFieldTest::new().await;
    let (params, _) = create_text_field(&test.grid_id());
    let field_count = test.field_count();
    let expected_field_count = field_count + 1;
    let scripts = vec![
        CreateField { params: params.clone() },
        AssertFieldCount(expected_field_count),
    ];
    test.run_scripts(scripts).await;
}

#[tokio::test]
async fn grid_update_field_with_empty_change() {
    let mut test = GridFieldTest::new().await;
    let (params, _) = create_single_select_field(&test.grid_id());
    let create_field_index = test.field_count();
    let scripts = vec![CreateField { params }];
    test.run_scripts(scripts).await;

    let field_rev = (&*test.field_revs.clone().pop().unwrap()).clone();
    let changeset = FieldChangesetParams {
        field_id: field_rev.id.clone(),
        grid_id: test.grid_id(),
        ..Default::default()
    };

    let scripts = vec![
        UpdateField { changeset },
        AssertFieldTypeOptionEqual {
            field_index: create_field_index,
            expected_type_option_data: field_rev.get_type_option_str(field_rev.ty).unwrap(),
        },
    ];
    test.run_scripts(scripts).await;
}

#[tokio::test]
async fn grid_update_field() {
    let mut test = GridFieldTest::new().await;
    let (params, _) = create_single_select_field(&test.grid_id());
    let scripts = vec![CreateField { params }];
    let create_field_index = test.field_count();
    test.run_scripts(scripts).await;
    //
    let single_select_field = (&*test.field_revs.clone().pop().unwrap()).clone();
    let mut single_select_type_option = SingleSelectTypeOptionPB::from(&single_select_field);
    single_select_type_option.options.push(SelectOptionPB::new("Unknown"));

    let changeset = FieldChangesetParams {
        field_id: single_select_field.id.clone(),
        grid_id: test.grid_id(),
        frozen: Some(true),
        width: Some(1000),
        ..Default::default()
    };

    // The expected_field must be equal to the field that applied the changeset
    let mut expected_field_rev = single_select_field.clone();
    expected_field_rev.frozen = true;
    expected_field_rev.width = 1000;
    expected_field_rev.insert_type_option(&single_select_type_option);

    let scripts = vec![
        UpdateField { changeset },
        AssertFieldFrozen {
            field_index: create_field_index,
            frozen: true,
        },
    ];
    test.run_scripts(scripts).await;
}

#[tokio::test]
async fn grid_delete_field() {
    let mut test = GridFieldTest::new().await;
    let original_field_count = test.field_count();
    let (params, _) = create_text_field(&test.grid_id());
    let scripts = vec![CreateField { params }];
    test.run_scripts(scripts).await;

    let text_field_rev = (&*test.field_revs.clone().pop().unwrap()).clone();
    let scripts = vec![
        DeleteField {
            field_rev: text_field_rev,
        },
        AssertFieldCount(original_field_count),
    ];
    test.run_scripts(scripts).await;
}

#[tokio::test]
async fn grid_switch_from_select_option_to_checkbox_test() {
    let mut test = GridFieldTest::new().await;
    let field_rev = test.get_first_field_rev(FieldType::SingleSelect);

    // Update the type option data of single select option
    let mut single_select_type_option = test.get_single_select_type_option(&field_rev.id);
    single_select_type_option.options.clear();
    // Add a new option with name CHECK
    single_select_type_option.options.push(SelectOptionPB {
        id: gen_option_id(),
        name: CHECK.to_string(),
        color: Default::default(),
    });
    // Add a new option with name UNCHECK
    single_select_type_option.options.push(SelectOptionPB {
        id: gen_option_id(),
        name: UNCHECK.to_string(),
        color: Default::default(),
    });

    let bytes: Bytes = single_select_type_option.try_into().unwrap();
    let scripts = vec![
        UpdateTypeOption {
            field_id: field_rev.id.clone(),
            type_option: bytes.to_vec(),
        },
        SwitchToField {
            field_id: field_rev.id.clone(),
            new_field_type: FieldType::Checkbox,
        },
    ];
    test.run_scripts(scripts).await;
}

#[tokio::test]
async fn grid_switch_from_checkbox_to_select_option_test() {
    let mut test = GridFieldTest::new().await;
    let field_rev = test.get_first_field_rev(FieldType::Checkbox).clone();
    let scripts = vec![SwitchToField {
        field_id: field_rev.id.clone(),
        new_field_type: FieldType::SingleSelect,
    }];
    test.run_scripts(scripts).await;

    let single_select_type_option = test.get_single_select_type_option(&field_rev.id);
    assert_eq!(single_select_type_option.options.len(), 2);
    assert!(single_select_type_option
        .options
        .iter()
        .any(|option| option.name == UNCHECK));
    assert!(single_select_type_option
        .options
        .iter()
        .any(|option| option.name == CHECK));
}
