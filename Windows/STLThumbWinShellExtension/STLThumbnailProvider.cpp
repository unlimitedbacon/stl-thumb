/******************************** Module Header ********************************\
Module Name:  STLThumbnailProvider.cpp
Project:      CppShellExtThumbnailHandler
Copyright (c) Microsoft Corporation.

The code sample demonstrates the C++ implementation of a thumbnail handler 
for a new file type registered with the .recipe extension. 

A thumbnail image handler provides an image to represent the item. It lets you 
customize the thumbnail of files with a specific file extension. Windows Vista 
and newer operating systems make greater use of file-specific thumbnail images 
than earlier versions of Windows. Thumbnails of 32-bit resolution and as large 
as 256x256 pixels are often used. File format owners should be prepared to 
display their thumbnails at that size. 

The example thumbnail handler implements the IInitializeWithStream and 
IThumbnailProvider interfaces, and provides thumbnails for .recipe files. 
The .recipe file type is simply an XML file registered as a unique file name 
extension. It includes an element called "Picture", embedding an image file. 
The thumbnail handler extracts the embedded image and asks the Shell to 
display it as a thumbnail.

This source is subject to the Microsoft Public License.
See http://www.microsoft.com/opensource/licenses.mspx#Ms-PL.
All other rights reserved.

THIS CODE AND INFORMATION IS PROVIDED "AS IS" WITHOUT WARRANTY OF ANY KIND, 
EITHER EXPRESSED OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE IMPLIED 
WARRANTIES OF MERCHANTABILITY AND/OR FITNESS FOR A PARTICULAR PURPOSE.
\*******************************************************************************/

#include "STLThumbnailProvider.h"
#include <Shlwapi.h>
#include <wincodec.h>
#include <stdio.h>
#include <wchar.h>
#include <cstdlib>

#pragma comment(lib, "Shlwapi.lib")
#pragma comment(lib, "windowscodecs.lib")


extern HINSTANCE g_hInst;
extern long g_cDllRef;


// Initialize member data
STLThumbnailProvider::STLThumbnailProvider()
	:
	m_cRef(1)
{
    InterlockedIncrement(&g_cDllRef);
}


// Tear down resources
STLThumbnailProvider::~STLThumbnailProvider()
{
    InterlockedDecrement(&g_cDllRef);
}


#pragma region IUnknown

// Query to the interface the component supported.
IFACEMETHODIMP STLThumbnailProvider::QueryInterface(REFIID riid, void **ppv)
{
    static const QITAB qit[] = 
    {
        QITABENT(STLThumbnailProvider, IThumbnailProvider),
        //QITABENT(STLThumbnailProvider, IInitializeWithStream), 
		QITABENT(STLThumbnailProvider, IInitializeWithFile),
        { 0 },
    };
    return QISearch(this, qit, riid, ppv);
}

// Increase the reference count for an interface on an object.
IFACEMETHODIMP_(ULONG) STLThumbnailProvider::AddRef()
{
	return InterlockedIncrement(&m_cRef);
}

// Decrease the reference count for an interface on an object.
IFACEMETHODIMP_(ULONG) STLThumbnailProvider::Release()
{
	ULONG cRef = InterlockedDecrement(&m_cRef);
	if (0 == cRef)
	{
		delete this;
	}

	return cRef;
}

#pragma endregion


//#pragma region IInitializeWithStream
//
//// Initializes the thumbnail handler with a stream.
//IFACEMETHODIMP STLThumbnailProvider::Initialize(IStream *pStream, DWORD grfMode)
//{
//    // A handler instance should be initialized only once in its lifetime. 
//    HRESULT hr = HRESULT_FROM_WIN32(ERROR_ALREADY_INITIALIZED);
//    if (m_pStream == NULL)
//    {
//        // Take a reference to the stream if it has not been initialized yet.
//        hr = pStream->QueryInterface(&m_pStream);
//    }
//    return hr;
//}
//
//#pragma endregion


#pragma region IInitializeWithFile

// Initializes the thumbnail handler with a file path.
IFACEMETHODIMP STLThumbnailProvider::Initialize(LPCWSTR pszFilePath, DWORD grfMode)
{
	stl_filename = pszFilePath;
	return S_OK;
}

#pragma endregion


#pragma region IThumbnailProvider

// Gets a thumbnail image and alpha type. The GetThumbnail is called with the 
// largest desired size of the image, in pixels. Although the parameter is 
// called cx, this is used as the maximum size of both the x and y dimensions. 
// If the retrieved thumbnail is not square, then the longer axis is limited 
// by cx and the aspect ratio of the original image respected. On exit, 
// GetThumbnail provides a handle to the retrieved image. It also provides a 
// value that indicates the color format of the image and whether it has 
// valid alpha information.
IFACEMETHODIMP STLThumbnailProvider::GetThumbnail(UINT cx, HBITMAP *phbmp,
	WTS_ALPHATYPE *pdwAlpha)
{
	HRESULT hr;

	// Create a temporary image file
	// TODO: Get stl-thumb to dump the image data directly to stdout,
	// then read the stream directly without making a temporary file
	// Also TODO: Dump a raw bitmap instead of wasting time compressing
	// a png.
	wchar_t lpTempPathBuffer[MAX_PATH];
	wchar_t image_filename[MAX_PATH];

	DWORD dwRetVal = GetTempPathW(MAX_PATH, lpTempPathBuffer);

	if (dwRetVal > MAX_PATH || (dwRetVal == 0))
	{
		hr = HRESULT_FROM_WIN32(GetLastError());
		return hr;
	}

	UINT uRetVal = GetTempFileName(lpTempPathBuffer,
		TEXT("stl"),
		0,
		image_filename);

	if (uRetVal == 0)
	{
		hr = HRESULT_FROM_WIN32(GetLastError());
		return hr;
	}

	// Create command to run
	// =====================

	// Get the stl-thumb executable path based on the location of this dll
	LPCWSTR exe_name = L"stl-thumb.exe";
	wchar_t dll_path[MAX_PATH];

	if (GetModuleFileName(g_hInst, dll_path, ARRAYSIZE(dll_path)) == 0)
	{
		hr = HRESULT_FROM_WIN32(GetLastError());
		return hr;
	}

	PathRemoveFileSpec(dll_path);

	wchar_t exe_path[MAX_PATH];
	PathCombine(exe_path, dll_path, exe_name);

	//LPCWSTR exe_path = L"C:\\Users\\Neo\\Desktop\\stl-thumb\\target\\debug\\stl-thumb.exe";
	//LPCWSTR image_filename = L"C:\\Users\\Neo\\Desktop\\cube.png";
	wchar_t command[MAX_PATH*3+6];

	swprintf_s(command, MAX_PATH * 3 + 6, L"\"%s\" -s %u \"%s\" \"%s\"", exe_path, cx, stl_filename, image_filename);

#ifdef _DEBUG
	// Open file for logging stl-thumb output
	//LPCWSTR log_path = L"C:\\Users\\Neo\\Desktop\\out.log";
	wchar_t log_path[MAX_PATH];
	PathCombine(log_path, dll_path, L"out.log");

	SECURITY_ATTRIBUTES sa;
	sa.nLength = sizeof(sa);
	sa.lpSecurityDescriptor = NULL;
	sa.bInheritHandle = TRUE;

	HANDLE h = CreateFile(log_path,
		FILE_APPEND_DATA,
		FILE_SHARE_WRITE | FILE_SHARE_READ,
		&sa,
		OPEN_ALWAYS,
		FILE_ATTRIBUTE_NORMAL,
		NULL);
#endif

	// Launch the process
	PROCESS_INFORMATION pi;
	STARTUPINFO si;
	BOOL ret = FALSE;
	DWORD flags = CREATE_NO_WINDOW;

	ZeroMemory(&pi, sizeof(PROCESS_INFORMATION));
	ZeroMemory(&si, sizeof(STARTUPINFO));
	si.cb = sizeof(STARTUPINFO);
	si.dwFlags |= STARTF_USESTDHANDLES;
	si.hStdInput = NULL;
#ifdef _DEBUG
	si.hStdError = h;
	si.hStdOutput = h;
#else
	si.hStdError = NULL;
	si.hStdOutput = NULL;
#endif

	ret = CreateProcess(exe_path,
		command,
		NULL,
		NULL,
		TRUE,
		flags,
		NULL,
		NULL,
		&si,
		&pi);
	
	if (!ret)
	{
		hr = HRESULT_FROM_WIN32(GetLastError());
		return hr;
	}

	WaitForSingleObject(pi.hProcess, INFINITE);

	// Clean up after execution
	DWORD exitCode = 0;
	GetExitCodeProcess(pi.hProcess, &exitCode);

	CloseHandle(pi.hProcess);
	CloseHandle(pi.hThread);
#ifdef _DEBUG
	CloseHandle(h);
#endif

	// Return here if stl-thumb.exe failed
	if (exitCode != 0) {
		DeleteFile(image_filename);
		return E_FAIL;
	}


	// Load the image it created
	// =========================

	// Create WIC factory
	IWICImagingFactory *pIWICFactory;

	hr = CoCreateInstance(
		CLSID_WICImagingFactory,
		nullptr,
		CLSCTX_INPROC_SERVER,
		IID_PPV_ARGS(&pIWICFactory)
	);

	if (FAILED(hr)) {
		return hr;
		DeleteFile(image_filename);
	}

	// Create image decoder
	IWICBitmapDecoder *pDecoder = NULL;

	hr = pIWICFactory->CreateDecoderFromFilename(
		LPCWSTR(image_filename),
		NULL,
		GENERIC_READ,
		WICDecodeMetadataCacheOnDemand,
		&pDecoder
	);

	if (FAILED(hr)) {
		pIWICFactory->Release();
		DeleteFile(image_filename);
		return hr;
	}

	// Load the first (only) frame from image
	IWICBitmapFrameDecode *pFrame = NULL;

	hr = pDecoder->GetFrame(0, &pFrame);

	if (FAILED(hr)) {
		pDecoder->Release();
		pIWICFactory->Release();
		DeleteFile(image_filename);
		return hr;
	}

	// Convert to 32bpp BGRA format w/ pre-multiplied alpha
	hr = ConvertBitmapSourceTo32bppHBITMAP(pFrame, pIWICFactory, phbmp);
	*pdwAlpha = WTSAT_ARGB;

	pFrame->Release();
	pDecoder->Release();
	pIWICFactory->Release();

	// Delete the temporary image file
	if (!DeleteFile(image_filename)) {
		return E_FAIL;
	}
	
	return hr;
}

#pragma endregion


HRESULT STLThumbnailProvider::ConvertBitmapSourceTo32bppHBITMAP(
	IWICBitmapSource *pBitmapSource, IWICImagingFactory *pImagingFactory,
	HBITMAP *phbmp)
{
	*phbmp = NULL;

	IWICBitmapSource *pBitmapSourceConverted = NULL;
	WICPixelFormatGUID guidPixelFormatSource;
	HRESULT hr = pBitmapSource->GetPixelFormat(&guidPixelFormatSource);

	if (SUCCEEDED(hr) && (guidPixelFormatSource != GUID_WICPixelFormat32bppBGRA))
	{
		IWICFormatConverter *pFormatConverter;
		hr = pImagingFactory->CreateFormatConverter(&pFormatConverter);
		if (SUCCEEDED(hr))
		{
			// Create the appropriate pixel format converter.
			hr = pFormatConverter->Initialize(pBitmapSource,
				GUID_WICPixelFormat32bppBGRA, WICBitmapDitherTypeNone, NULL,
				0, WICBitmapPaletteTypeCustom);
			if (SUCCEEDED(hr))
			{
				hr = pFormatConverter->QueryInterface(&pBitmapSourceConverted);
			}
			pFormatConverter->Release();
		}
	}
	else
	{
		// No conversion is necessary.
		hr = pBitmapSource->QueryInterface(&pBitmapSourceConverted);
	}

	if (SUCCEEDED(hr))
	{
		UINT nWidth, nHeight;
		hr = pBitmapSourceConverted->GetSize(&nWidth, &nHeight);
		if (SUCCEEDED(hr))
		{
			BITMAPINFO bmi = { sizeof(bmi.bmiHeader) };
			bmi.bmiHeader.biWidth = nWidth;
			bmi.bmiHeader.biHeight = -static_cast<LONG>(nHeight);
			bmi.bmiHeader.biPlanes = 1;
			bmi.bmiHeader.biBitCount = 32;
			bmi.bmiHeader.biCompression = BI_RGB;

			BYTE *pBits;
			HBITMAP hbmp = CreateDIBSection(NULL, &bmi, DIB_RGB_COLORS,
				reinterpret_cast<void **>(&pBits), NULL, 0);
			hr = hbmp ? S_OK : E_OUTOFMEMORY;
			if (SUCCEEDED(hr))
			{
				WICRect rect = { 0, 0, nWidth, nHeight };

				// Convert the pixels and store them in the HBITMAP.  
				// Note: the name of the function is a little misleading - 
				// we're not doing any extraneous copying here.  CopyPixels 
				// is actually converting the image into the given buffer.
				hr = pBitmapSourceConverted->CopyPixels(&rect, nWidth * 4,
					nWidth * nHeight * 4, pBits);
				if (SUCCEEDED(hr))
				{
					*phbmp = hbmp;
				}
				else
				{
					DeleteObject(hbmp);
				}
			}
		}
		pBitmapSourceConverted->Release();
	}
	return hr;
}