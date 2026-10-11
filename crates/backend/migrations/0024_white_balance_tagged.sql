UPDATE edits SET edits_json = json_set(edits_json, '$.masks', (
    SELECT json_group_array(
        CASE WHEN json_type(value, '$.edits.wb_tint') IN ('integer', 'real')
            THEN json_set(value, '$.edits.wb_tint', 0.0 - json_extract(value, '$.edits.wb_tint'))
            ELSE json(value)
        END ORDER BY key)
    FROM json_each(edits_json, '$.masks')))
WHERE json_type(edits_json, '$.masks') = 'array'
    AND (json_type(edits_json, '$.basic.wb_temp') IS NOT NULL
        OR json_type(edits_json, '$.basic.wb_tint') IS NOT NULL);

UPDATE edits SET edits_json = json_set(
    json_remove(edits_json, '$.basic.wb_temp', '$.basic.wb_tint'),
    '$.basic.white_balance',
    json_object(
        'mode', 'relative',
        'temp', coalesce(json_extract(edits_json, '$.basic.wb_temp'), 0.0),
        'tint', 0.0 - coalesce(json_extract(edits_json, '$.basic.wb_tint'), 0.0)))
WHERE json_type(edits_json, '$.basic.wb_temp') IS NOT NULL
    OR json_type(edits_json, '$.basic.wb_tint') IS NOT NULL;

UPDATE edits_history SET edits_json = json_set(edits_json, '$.masks', (
    SELECT json_group_array(
        CASE WHEN json_type(value, '$.edits.wb_tint') IN ('integer', 'real')
            THEN json_set(value, '$.edits.wb_tint', 0.0 - json_extract(value, '$.edits.wb_tint'))
            ELSE json(value)
        END ORDER BY key)
    FROM json_each(edits_json, '$.masks')))
WHERE json_type(edits_json, '$.masks') = 'array'
    AND (json_type(edits_json, '$.basic.wb_temp') IS NOT NULL
        OR json_type(edits_json, '$.basic.wb_tint') IS NOT NULL);

UPDATE edits_history SET edits_json = json_set(
    json_remove(edits_json, '$.basic.wb_temp', '$.basic.wb_tint'),
    '$.basic.white_balance',
    json_object(
        'mode', 'relative',
        'temp', coalesce(json_extract(edits_json, '$.basic.wb_temp'), 0.0),
        'tint', 0.0 - coalesce(json_extract(edits_json, '$.basic.wb_tint'), 0.0)))
WHERE json_type(edits_json, '$.basic.wb_temp') IS NOT NULL
    OR json_type(edits_json, '$.basic.wb_tint') IS NOT NULL;
