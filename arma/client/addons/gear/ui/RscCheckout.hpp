class RscButtonMenuOk;
class RscButtonMenuCancel;
class RscText;
class RscPicture;

#define OFFSET -6
#define WIDTH 50

class GVAR(RscCheckout) {
    idd = IDD_RSCDISPLAYCHECKOUT;
    name = QGVAR(RscCheckout);
    onLoad = QUOTE(_this call FUNC(shop_checkout_onLoad));
    class controls {
        class Background: RscPicture {
            text = "#(argb,8,8,3)color(0,0,0,0.8)";
            x = QUOTE(X_PART((OFFSET + 1.5)));
            y = QUOTE(Y_PART(2.5));
            w = QUOTE(W_PART(WIDTH));
            h = QUOTE(H_PART(20));
        };
        class Background2: RscPicture {
            text = "#(argb,8,8,3)color(0,0,0,0.8)";
            x = QUOTE(X_PART((OFFSET + 1.5)));
            y = QUOTE(Y_PART(2.5));
            w = QUOTE(W_PART(WIDTH));
            h = QUOTE(H_PART(20));
        };
        class RscText_1001: RscText {
            idc = 1001;
            text = "Confirm Purchase"; //--- ToDo: Localize;
            x = QUOTE(X_PART((OFFSET + 1.5)));
            y = QUOTE(Y_PART(1));
            w = QUOTE(W_PART(WIDTH));
            h = QUOTE(H_PART(1.5));
            colorBackground[] = {0.13,0.54,0.21,0.8};
        };
        class GVAR(Buy): RscButtonMenuOk {
            text = "Purchase";
            x = QUOTE(X_PART((OFFSET + (WIDTH - 4))));
            y = QUOTE(Y_PART(23));
            w = QUOTE(W_PART(5.5));
            h = QUOTE(H_PART(1));
            colorBackground[] = {0,0,0,0.8};
        };
        class GVAR(Header): RscListNBox {
            idc = IDC_RSCDISPLAYCHECKOUT_HEADER;
            x = QUOTE(X_PART((OFFSET + 1.5)));
            y = QUOTE(Y_PART(2.5));
            w = QUOTE(W_PART(WIDTH));
            h = QUOTE(H_PART(1));
            columns[] = {-0.01, 0.15, 0.08, 0.6, 0.7, 0.8, 0.9};
            colorBackground[] = {0.13,0.54,0.21,0.8};
        };
        class Items: GVAR(Header) {
            idc = IDC_RSCDISPLAYCHECKOUT_ITEMS;
            x = QUOTE(X_PART((OFFSET + 1.5)));
            y = QUOTE(Y_PART(3.5));
            h = QUOTE(H_PART(19));
        };
        class GVAR(Cancel): RscButtonMenuCancel {
            x = QUOTE(X_PART((OFFSET + 1.5)));
            y = QUOTE(Y_PART(23));
            w = QUOTE(W_PART(5.5));
            h = QUOTE(H_PART(1));
            colorBackground[] = {0,0,0,0.8};
        };
    };
};
