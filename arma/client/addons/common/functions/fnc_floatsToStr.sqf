private _floats = _this;

if (_floats isEqualType 1.0) exitWith {
    _floats toFixed 11
};

private _strs = [];
{
    if (_x isEqualType []) then {
        _strs pushBack (_x call EFUNC(common,floatsToStr));
    } else {
        if (_x isEqualType 1.0) then {
            _strs pushBack (_x toFixed 11);
        } else {
            _strs pushBack _x;
        };
    };
} forEach _floats;

_strs
