#include "script_component.hpp"

params ["_func", "_data"];

switch (_func) do {
    case "items:clear": {
        GVAR(shop_items_importing) = createHashMap;
    };
    case "items:set": {
        (parseSimpleArray _data) params ["_class", "_entry", "_pretty"];
        GVAR(shop_items_importing) set [_class, _entry];

        private _newPretty = getText (configFile >> "CfgWeapons" >> _class >> "displayName");
        if (_newPretty == "") then {
            _newPretty = getText (configFile >> "CfgVehicles" >> _class >> "displayName");
        };
        if (_newPretty == "") then {
            _newPretty = getText (configFile >> "CfgMagazines" >> _class >> "displayName");
        };
        if (_newPretty != "") then {
            if (_pretty == _newPretty) exitWith {};
            EXTCALL("gear:shop:pretty",[ARR_2(_class,_newPretty)]);
        };
    };
    case "items:publish": {
        GVAR(shop_items) = +GVAR(shop_items_importing);
        publicVariable QGVAR(shop_items);
        INFO_1("Published shop items: %1",count GVAR(shop_items));
    };
    case "items:err": {
        ERROR("Error importing shop items");
    };

    case "enter:ok": {
        (parseSimpleArray _data) params ["_steam", "_locker", "_balance"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(shop_enter_ok), [_locker, _balance], [_player]] call CBA_fnc_targetEvent;
    };
    case "enter:err": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(shop_enter_err), [], [_player]] call CBA_fnc_targetEvent;
    };
    case "leave:ok": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(shop_leave_ok), [], [_player]] call CBA_fnc_targetEvent;
    };
    case "leave:err": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(shop_leave_err), [], [_player]] call CBA_fnc_targetEvent;
    };

    case "purchase:ok": {
        (parseSimpleArray _data) params ["_steam", "_locker", "_balance"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(shop_purchase_ok), [_locker, _balance], [_player]] call CBA_fnc_targetEvent;
    };
    case "purchase:err": {
        (parseSimpleArray _data) params ["_steam"];
        private _player = [_steam] call EFUNC(common,playerFromSteam);
        [QGVAR(shop_purchase_err), [], [_player]] call CBA_fnc_targetEvent;
    };
};
