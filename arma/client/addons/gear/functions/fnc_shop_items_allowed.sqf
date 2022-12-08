#include "script_component.hpp"

params [
    ["_items", createHashMap, [createHashMap]]
];

private _ret = [];

private _my_roles = player getVariable [QEGVAR(discord,roles), []];

{
    private _roles = [_x] call FUNC(shop_item_roles);
    if (_roles isEqualTo [] || {count (_my_roles arrayIntersect _roles) > 0}) then {
        // Check for DLC ownership
        private _weapon = getAssetDLCInfo [_x, configFile >> "CfgWeapons"];
        private _owned = if !(_weapon#3) then {
            private _vehicle = getAssetDLCInfo [_x, configFile >> "CfgVehicles"];
            if (_vehicle#3) then {
                _vehicle#1
            } else {
                true
            }
        } else {
            _weapon#1
        };

        if (_owned) then {
            _ret pushBack _x;
        };
    };
} forEach _items;

_ret
