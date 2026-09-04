#include <windows.h>
#include <shobjidl.h>
#include <propvarutil.h>
#include <iostream>
#include <string>

#pragma comment(lib, "ole32.lib")
#pragma comment(lib, "shell32.lib")

int main(int argc, char* argv[]) {
    if (argc < 2) {
        std::cout << "Usage: get_pdf_pages.exe <pdf_path>" << std::endl;
        return 1;
    }

    std::wstring path(argv[1]);

    CoInitializeEx(NULL, COINIT_APARTMENTTHREADED);

    IShellLibrary* pLibrary = NULL;
    HRESULT hr = SHCreateLibrary(IID_PPV_ARGS(&pLibrary));

    if (SUCCEEDED(hr)) {
        IShellItem* pItem = NULL;
        hr = SHCreateItemFromParsingName(path.c_str(), NULL, IID_PPV_ARGS(&pItem));

        if (SUCCEEDED(hr)) {
            IPropertyStore* pStore = NULL;
            hr = pItem->BindToHandler(NULL, BHID_PropertyStore, IID_PPV_ARGS(&pStore));

            if (SUCCEEDED(hr)) {
                PROPERTYKEY key;
                key.fmtid = { 0xeec05d88, 0xc63c, 0x41be, { 0x8b, 0xf9, 0xad, 0x85, 0x65, 0x3b, 0x1b, 0x5b }; // System.Pages
                key.pid = 2;

                PROPVARIANT prop;
                hr = pStore->GetValue(key, &prop);

                if (SUCCEEDED(hr) && prop.vt == VT_UI4) {
                    std::cout << "Pages: " << prop.ulVal << std::endl;
                }

                PropVariantClear(&prop);
                pStore->Release();
            }
            pItem->Release();
        }
        pLibrary->Release();
    }

    CoUninitialize();
    return 0;
}
