#include "script_component.hpp"

params ["_args"];
_args params ["_display"];

private _btnHide = _display displayCtrl IDC_buttonHide;
private _btnClose = _display displayCtrl IDC_menuBarClose;

private _loadout = [player] call CBA_fnc_getLoadout;
private _items = [_loadout] call FUNC(loadout_items);

private _invalid = [_items] call FUNC(shop_items_purchasable);
if (_invalid isNotEqualTo []) exitWith {
    _btnHide ctrlEnable false;
    _btnHide ctrlSetText "Invalid Items";
    private _names = [];
    {
        private _config = (_x call CBA_fnc_getItemConfig);
        private _name = getText (_config >> "displayName");
        _names pushBack _name;
    } forEach _invalid;
    _btnHide ctrlSetTooltip (_names joinString "\n");
    if !(GVAR(readOnly) || EGVAR(campaigns,loadouts)) then {
        _btnClose ctrlSetText "Cancel";
    };
};

private _cost = [_items] call FUNC(shop_items_cost);

if (GVAR(readOnly) || EGVAR(campaigns,loadouts)) exitWith {
    _btnHide ctrlEnable false;
    _btnHide ctrlSetText format ["%1", _cost];
    _btnHide ctrlSetTooltip "Loadout Cost";
};

if (_cost == 0) then {
    _btnHide ctrlEnable false;
    _btnHide ctrlSetText format ["0 / %1", GVAR(shop_balance)];
    _btnHide ctrlSetTooltip "Current PMC Balance";
    _btnClose ctrlSetText "Apply";
} else {
    if (GVAR(shop_processing)) then {
        _btnHide ctrlEnable false;
        _btnHide ctrlSetText "Processing";
    } else {
        _btnHide ctrlEnable true;
        private _cost = [_items, 0] call FUNC(shop_items_cost);
        _btnHide ctrlSetText format ["%1 / %2", _cost, GVAR(shop_balance)];
        _btnHide ctrlSetTooltip "Buy Current Gear";
        _btnClose ctrlSetText "Cancel";
    };
};
