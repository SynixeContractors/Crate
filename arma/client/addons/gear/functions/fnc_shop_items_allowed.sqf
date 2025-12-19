#include "script_component.hpp"

params [
    ["_items", [], [[]]]
];

private _ret = [];

private _my_roles = player getVariable [QEGVAR(discord,roles), []];

{
    private _roles = [_x] call FUNC(shop_item_roles);
    if (_roles isEqualTo [] || {count (_my_roles arrayIntersect _roles) > 0}) then {
        if ([_x] call EFUNC(common,isDLCOwned)) then {
            _ret pushBack _x;
        };
    };
} forEach _items;

private _suppressors = missionNamespace getVariable [QGVAR(suppressors_allowed), false];
_ret = _ret select {
    private _type = getNumber (configFile >> "CfgWeapons" >> _x >> "ItemInfo" >> "type");
    if (_type == 101) then {
        _suppressors
    } else {
        true
    }
};

_ret
