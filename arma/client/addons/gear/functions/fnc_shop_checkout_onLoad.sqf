#include "script_component.hpp"

params ["_display"];

_display displayAddEventHandler ["Unload", {
    [findDisplay 1127001] call ace_arsenal_fnc_buttonHide;
}];

[findDisplay 1127001] call ace_arsenal_fnc_buttonHide;

private _header = _display displayCtrl IDC_RSCDISPLAYCHECKOUT_HEADER;
_header lnbAddRow ["", "Item", "#", "Price", "Total"];
_header ctrlEnable false;

private _list = _display displayCtrl IDC_RSCDISPLAYCHECKOUT_ITEMS;

private _loadout = [player] call CBA_fnc_getLoadout;
private _items = [_loadout] call FUNC(loadout_items);
{
    _x params ["_item", "_price", "_need", "_total", "_global"];
    if (_price != 0) then {
        private _config = _item call CBA_fnc_getItemConfig;
        private _name = getText (_config >> "displayName");
        private _index = _list lnbAddRow [
            "",
            _name,
            format ["%1", _need],
            if (_global) then { format ["%1 (Global)", _price] } else { format ["%1", _price] },
            format ["%1", _total]
        ];
        _list lnbSetPicture [[_index, 0], getText (_config >> "picture")];
    };
} forEach ([_items] call FUNC(shop_items_difference));

private _fnc_onConfirm = {
    params [["_ctrlButtonOK", controlNull, [controlNull]]];
    private _display = ctrlParent _ctrlButtonOK;
    if (isNull _display) exitWith {};

    private _loadout = [player] call CBA_fnc_getLoadout;
    private _items = [_loadout] call FUNC(loadout_items);
    [[_items] call FUNC(shop_items_removeOwned)] call FUNC(shop_items_purchase);
};

private _cost = [_items, 0] call FUNC(shop_items_cost);
if (_cost > GVAR(shop_balance)) then {
    (_display displayCtrl 1) ctrlEnable false;
    (_display displayCtrl 1001) ctrlSetText "Insufficient Funds";
} else {
    (_display displayCtrl 1) ctrlAddEventHandler ["ButtonClick", _fnc_onConfirm];
    private _personal = [_items, 0] call FUNC(shop_items_cost);
    private _global = [_items, 1] call FUNC(shop_items_cost);
    (_display displayCtrl 1001) ctrlSetText format ["$%1 ($%2 Global)", _personal, _global];
};
