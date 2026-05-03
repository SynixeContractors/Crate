#include "..\script_component.hpp"

params [
    ["_items"]
];

if (typeName _items != "HASHMAP") exitWith {
    WARNING("shop_items_allowed: items is not a hashmap");
    []
};

private _ret = [];

private _my_roles = player getVariable [QEGVAR(discord,roles), []];
private _keys = keys _items;

{
    private _entry = _items get _x;
    if !(_entry select 2) then { continue };

    private _roles = [_x] call FUNC(shop_item_roles);
    if (_roles isEqualTo [] || {(_my_roles arrayIntersect _roles) isNotEqualTo []}) then {
        if ([_x] call EFUNC(common,isDLCOwned)) then {
            _ret pushBack _x;
        };
    };
} forEach _keys;

private _suppressors = missionNamespace getVariable [QGVAR(suppressors_allowed), false];
_ret = _ret select {
    private _type = getNumber (configFile >> "CfgWeapons" >> _x >> "ItemInfo" >> "type");
    [true, _suppressors] select (_type == 101)
};

_ret
