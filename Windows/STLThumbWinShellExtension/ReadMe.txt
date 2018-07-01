=============================================================================
    DYNAMIC LINK LIBRARY : CppShellExtThumbnailHandler Project Overview
=============================================================================

/////////////////////////////////////////////////////////////////////////////
Summary:

The code sample demonstrates the C++ implementation of a thumbnail handler 
for a new file type registered with the .recipe extension. 

A thumbnail image handler provides an image to represent the item. It lets 
you customize the thumbnail of files with a specific file extension. Windows 
Vista and newer operating systems make greater use of file-specific thumbnail 
images than earlier versions of Windows. Thumbnails of 32-bit resolution and 
as large as 256x256 pixels are often used. File format owners should be 
prepared to display their thumbnails at that size. 

The example thumbnail handler has the class ID (CLSID): 
    {4D2FBA8D-621B-4447-AF6D-5794F479C4A5}

The handler implements the IInitializeWithStream and IThumbnailProvider 
interfaces, and provides thumbnails for .recipe files. The .recipe file type 
is simply an XML file registered as a unique file name extension. It includes 
an element called "Picture", embedding an image file. The thumbnail handler 
extracts the embedded image and asks the Shell to display it as a thumbnail.


/////////////////////////////////////////////////////////////////////////////
Prerequisite:

The example thumbnail handler must be registered on Windows Vista or newer 
operating systems.


/////////////////////////////////////////////////////////////////////////////
Setup and Removal:

A. Setup

If you are going to use the Shell extension in a x64 Windows system, please 
configure the Visual C++ project to target 64-bit platforms using project 
configurations (http://msdn.microsoft.com/en-us/library/9yb4317s.aspx). 

If the extension is to be loaded in a 32-bit Windows system, you can use the 
default Win32 project configuration to build the project.

In a command prompt running as administrator, navigate to the folder that 
contains the build result CppShellExtThumbnailHandler.dll and enter the 
command:

    Regsvr32.exe CppShellExtThumbnailHandler.dll

The thumbnail handler is registered successfully if you see a message box 
saying:

    "DllRegisterServer in CppShellExtThumbnailHandler.dll succeeded."

B. Removal

In a command prompt running as administrator, navigate to the folder that 
contains the build result CppShellExtThumbnailHandler.dll and enter the 
command:

    Regsvr32.exe /u CppShellExtThumbnailHandler.dll

The thumbnail handler is unregistered successfully if you see a message box 
saying:

    "DllUnregisterServer in CppShellExtThumbnailHandler.dll succeeded."


/////////////////////////////////////////////////////////////////////////////
Demo:

The following steps walk through a demonstration of the thumbnail handler 
code sample.

Step1. If you are going to use the Shell extension in a x64 Windows system, 
please configure the Visual C++ project to target 64-bit platforms using 
project configurations (http://msdn.microsoft.com/en-us/library/9yb4317s.aspx). 
If the extension is to be loaded in a 32-bit Windows system, you can use the 
default Win32 project configuration.

Step2. After you successfully build the sample project in Visual Studio 2010, 
you will get a DLL: CppShellExtThumbnailHandler.dll. Start a command prompt 
as administrator, navigate to the folder that contains the file and enter the 
command:

    Regsvr32.exe CppShellExtThumbnailHandler.dll

The thumbnail handler is registered successfully if you see a message box 
saying:

    "DllRegisterServer in CppShellExtThumbnailHandler.dll succeeded."

Step3. Find the chocolatechipcookies.recipe file in the sample folder. You 
will see a picture of chocoate chip cookies as its thumbnail. 

The .recipe file type is simply an XML file registered as a unique file name 
extension. It includes an element called "Picture", embedding an image file. 
The thumbnail handler extracts the embedded image and asks the Shell to 
display it as a thumbnail. 

Step4. In the same command prompt, run the command 

    Regsvr32.exe /u CppShellExtThumbnailHandler.dll

to unregister the Shell thumbnail handler.


/////////////////////////////////////////////////////////////////////////////
Implementation:

A. Creating and configuring the project

In Visual Studio 2010, create a Visual C++ / Win32 / Win32 Project named 
"CppShellExtThumbnailHandler". In the "Application Settings" page of Win32 
Application Wizard, select the application type as "DLL" and check the "Empty 
project" option. After you click the Finish button, an empty Win32 DLL 
project is created.

-----------------------------------------------------------------------------

B. Implementing a basic Component Object Model (COM) DLL

Shell extension handlers are COM objects implemented as DLLs. Making a basic 
COM includes implementing DllGetClassObject, DllCanUnloadNow, 
DllRegisterServer, and DllUnregisterServer in (and exporting them from) the 
DLL, adding a COM class with the basic implementation of the IUnknown 
interface, preparing the class factory for your COM class. The relevant files 
in this code sample are:

  dllmain.cpp - implements DllMain and the DllGetClassObject, DllCanUnloadNow, 
    DllRegisterServer, DllUnregisterServer functions that are necessary for a 
    COM DLL. 

  GlobalExportFunctions.def - exports the DllGetClassObject, DllCanUnloadNow, 
    DllRegisterServer, DllUnregisterServer functions from the DLL through the 
    module-definition file. You need to pass the .def file to the linker by 
    configuring the Module Definition File property in the project's Property 
    Pages / Linker / Input property page.

  Reg.h/cpp - defines the reusable helper functions to register or unregister 
    in-process COM components in the registry: 
    RegisterInprocServer, UnregisterInprocServer

  RecipeThumbnailProvider.h/cpp - defines the COM class. You can find the basic 
    implementation of the IUnknown interface in the files.

  ClassFactory.h/cpp - defines the class factory for the COM class. 

-----------------------------------------------------------------------------

C. Implementing the thumbnail handler and registering it for a certain file 
class

-----------
Implementing the thumbnail handler:

The RecipeThumbnailProvider.h/cpp files define a thumbnail provider. It 
implements the IInitializeWithStream and IThumbnailProvider interfaces, and 
provides thumbnails for .recipe files. 

    class RecipeThumbnailProvider : 
        public IInitializeWithStream, 
        public IThumbnailProvider
    {
    public:
        // IInitializeWithStream
        IFACEMETHODIMP Initialize(IStream *pStream, DWORD grfMode);

        // IThumbnailProvider
        IFACEMETHODIMP GetThumbnail(UINT cx, HBITMAP *phbmp, WTS_ALPHATYPE *pdwAlpha);
    };

  1. Implementing IThumbnailProvider
  
  The IThumbnailProvider interface has been introduced in Windows Vista to 
  make providing a thumbnail easier and more straightforward than in the past, 
  when IExtractImage would have been used instead. Note, that existing code 
  that uses IExtractImage is still valid under Windows Vista. However, 
  IExtractImage is not supported in the Details pane. 

  IThumbnailProvider has only one method¡ªGetThumbnail¡ªthat is called with the 
  largest desired size of the image, in pixels. Although the parameter is 
  called cx, this is used as the maximum size of both the x and y dimensions. 
  If the retrieved thumbnail is not square, then the longer axis is limited 
  by cx and the aspect ratio of the original image respected.

  On exit, GetThumbnail provides a handle to the retrieved image. It also 
  provides a value that indicates the color format of the image and whether 
  it has valid alpha information.

    IFACEMETHODIMP RecipeThumbnailProvider::GetThumbnail(UINT cx, HBITMAP *phbmp, 
        WTS_ALPHATYPE *pdwAlpha)
    {
        // Load the XML document.
        IXMLDOMDocument *pXMLDoc = NULL;
        HRESULT hr = LoadXMLDocument(&pXMLDoc);
        if (SUCCEEDED(hr))
        {
            // Read the preview image from the XML document.
            hr = GetRecipeImage(pXMLDoc, cx, phbmp, pdwAlpha);
            pXMLDoc->Release();
        }
        return hr;
    }

  The .recipe file type is simply an XML file registered as a unique file 
  name extension. It includes an element called Picture that embeds images 
  to be used as the thumbnail for this particular .recipe file. The XML 
  may provide images of different sizes, and the code can query image 
  matching the desired size specified by the cx parameter of GetThumbnail. 
  For simplicity, this sample omits the cx paramter and provides only one 
  image for all situations.

  2. Implementing IInitializeWithStream/IInitializeWithItem/IInitializeWithFile

  IThumbnailProvider must always be implemented in concert with one of these 
  interfaces: 
  
    IInitializeWithStream - provides the file stream
    IInitializeWithItem - provides the IShellItem
    IInitializeWithFile - provides the file path

  Whenever possible, it is recommended that initialization be done through a 
  stream using IInitializeWithStream. Benefits of this include increased 
  security and stability.

    IStream *m_pStream;

    IFACEMETHODIMP RecipeThumbnailProvider::Initialize(IStream *pStream, DWORD grfMode)
    {
        // A handler instance should be initialized only once in its lifetime. 
        HRESULT hr = HRESULT_FROM_WIN32(ERROR_ALREADY_INITIALIZED);
        if (m_pStream == NULL)
        {
            // Take a reference to the stream if it has not been initialized yet.
            hr = pStream->QueryInterface(&m_pStream);
        }
        return hr;
    }

-----------
Registering the handler for a certain file class:

The CLSID of the handler is declared at the beginning of dllmain.cpp.

// {4D2FBA8D-621B-4447-AF6D-5794F479C4A5}
const CLSID CLSID_RecipeThumbnailProvider = 
{ 0x4D2FBA8D, 0x621B, 0x4447, { 0xAF, 0x6D, 0x57, 0x94, 0xF4, 0x79, 0xC4, 0xA5 } };

When you write your own handler, you must create a new CLSID by using the 
"Create GUID" tool in the Tools menu, and specify the CLSID value here.

Thumbnail handlers can be associated with a file class. The handlers are 
registered by setting the default value of the following registry key to be 
the CLSID the handler class. 

    HKEY_CLASSES_ROOT\<File Type>\shellex\{e357fccd-a995-4576-b01f-234630154e96}

The registration of the thumbnail handler is implemented in the 
DllRegisterServer function of dllmain.cpp. DllRegisterServer first calls the 
RegisterInprocServer function in Reg.h/cpp to register the COM component. 
Next, it calls RegisterShellExtThumbnailHandler to associate the handler 
with a certain file type. If the file type starts with '.', it tries to read 
the default value of the HKCR\<File Type> key which may contain the Program 
ID to which the file type is linked. If the default value is not empty, use 
the Program ID as the file type to proceed the registration. 

For example, this code sample associates the handler with '.recipe' files. 
The following keys and values are added in the registration process of the 
sample handler. 

    HKCR
    {
        NoRemove CLSID
        {
            ForceRemove {4D2FBA8D-621B-4447-AF6D-5794F479C4A5} = 
                s 'CppShellExtThumbnailHandler.RecipeThumbnailProvider Class'
            {
                InprocServer32 = s '<Path of CppShellExtThumbnailHandler.DLL file>'
                {
                    val ThreadingModel = s 'Apartment'
                }
            }
        }
        NoRemove .recipe
        {
            NoRemove shellex
            {
                {e357fccd-a995-4576-b01f-234630154e96} = 
                    s '{4D2FBA8D-621B-4447-AF6D-5794F479C4A5}'
            }
        }
    }

The unregistration is implemented in the DllUnregisterServer function of 
dllmain.cpp. It removes the HKCR\CLSID\{<CLSID>} key and the 
HKCR\<File Type>\shellex\{e357fccd-a995-4576-b01f-234630154e96} key.


/////////////////////////////////////////////////////////////////////////////
Diagnostic:

Debugging thumbnail handlers is difficult for several reasons.

1) The Windows Explorer hosts thumbnail providers in an isolated process to 
get robustness and improve security. Because of this it is difficult to debug 
your handler as you cannot set breakpoints on your code in the explorer.exe 
process as it is not loaded there. The isolated process is DllHost.exe and 
this is used for other purposes so finding the right instance of this process 
is difficult. 

2) Once a thumbnail is computed for a particular file it is cached and your 
handler won't be called again for that item unless you invalidate the cache 
by updating the modification date of the file. Note that this cache works 
even if the files are renamed or moved.

Given all of these issues the easiest way to debug your code in a test 
application then once you have proven it works there test it in the context 
of the explorer. 

Another thing to do is to disable the process isolation feature of explorer. 
You can do this by putting the following named value on the CLSID of your 
handler

    HKCR\CLSID\{CLSID of Your Handler}
        DisableProcessIsolation=REG_DWORD:1

Be sure to not ship your handler with this on as customers require the 
security and robustness benefits of the isolated process feature.


/////////////////////////////////////////////////////////////////////////////
References:

MSDN: Thumbnail Handlers
http://msdn.microsoft.com/en-us/library/cc144118.aspx

MSDN: Building Thumbnail Handlers
http://msdn.microsoft.com/en-us/library/cc144114.aspx

MSDN: Thumbnail Handler Guidelines
http://msdn.microsoft.com/en-us/library/cc144115.aspx

MSDN: IThumbnailProvider Interface
http://msdn.microsoft.com/en-us/library/bb774614.aspx


/////////////////////////////////////////////////////////////////////////////