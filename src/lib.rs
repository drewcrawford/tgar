/*!

![logo](art/logo.png)

This is the world's simplest way to write images from Rust in the TGA format. It is one single file and has no dependencies.

If you are looking for a drop-dead simple way to write images to open in your image editor, and have a tab open on BMP and TIFF formats right now trying to figure out which one you can emit using a for loop, this is the crate you are looking for. TGA is a widely supported but trivial file format. This library supports BGRA 32-bit uncompressed images only, a popular and trivial subset.

I designed this library to dump GPU textures for debugging purposes, but it is suitable for a wide variety of applications.

*/
#[repr(C)]
struct ImageSpecification {
    x_origin: u16,
    y_origin: u16,
    image_width: u16,
    image_height: u16,
    ///bits per pixel
    pixel_depth: u8,
    ///bits 3-0 give the alpha channel depth, bits 5-4 give pixel ordering
    image_descriptor: u8
}
#[repr(C)]
struct Header {
    ///length of image ID field
    id_length: u8,
    ///whether a color map is included
    color_map_type: u8,
    ///compression and color types
    image_type: u8,
    ///describes the color map
    color_map_specification: [u8; 5],
    image_specification: ImageSpecification,
    image_id: [u8; IMAGE_ID.len()],
}
//http://www.paulbourke.net/dataformats/tga/
const IMAGE_ID: [u8; 17] = *b"drewcrawford/tgar";
impl Header {
    ///Creates a header for a texture with the associated with/height in bgra format.
    fn new_bgra(width: u16, height: u16) -> Header {
        Header {
            id_length:  IMAGE_ID.len() as u8,
            color_map_type: 0, //no color map
            image_type: 2, //uncompressed true-color image
            color_map_specification: [0,0,0,0,0],
            image_specification: ImageSpecification {
                x_origin: 0,
                y_origin: 0,
                image_width: width.to_le(),
                image_height: height.to_le(),
                pixel_depth: 32, //bpp, RGBA
                image_descriptor: 0b1000 | //bits 3-0 are the "number of attribute bits associated with each pixel", for TGA 32
                                            //this should be 8
                                //bit 4 must be 0
                                //bit 5 - 0=>lower left origin, 1=> upper left origin
                                0b1 << 5 |
                                0, //bits 7-8 'interleaving' flag.  We specify we don't interleave, I guess.
            },
            image_id: IMAGE_ID.clone(),
            //no color map data
            //image data to follow
        }
    }
}
/**
A BGRA pixel format.

# Memory layout

Defined to be the 32-bit BGRA pixel.  Each b,g,r,a component is a u8.
*/
#[repr(C)]
#[derive(Debug,Clone,Copy,PartialEq,Eq,Hash)]
pub struct PixelBGRA {
    //seems to be BGRA in practice
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

/**
An image encoded in TGA format.  32-bit, BGRA data.
*/
#[derive(Debug,Clone,PartialEq,Eq,Hash)]
pub struct BGRA {
    data: Box<[u8]>,
}

impl BGRA {
    ///Converts the type into boxed data.
    pub fn into_data(self) -> Box<[u8]> {
        self.data
    }
    ///Encodes a new image.
    ///
    /// The size of the data must match the provided width and height.
    pub fn new(width: u16, height: u16, data: &[PixelBGRA]) -> BGRA {
        //header + data
        let allocation_size = core::mem::size_of::<Header>() + core::mem::size_of::<PixelBGRA>() * data.len();
        let data_offset = core::mem::size_of::<Header>() - 1; //because index 0 is written to, lol.  fucking oboes
        assert!(width as usize * height as usize == data.len());
        let mut buf = Vec::with_capacity(allocation_size);
        unsafe{
            let header = Header::new_bgra(width, height);
            let header_ptr = buf.as_mut_ptr() as *mut Header;
            header_ptr.write(header);
            //we promise to fill this size!
            buf.set_len(allocation_size);
            let mut body_ptr = &mut buf[data_offset] as *mut u8 as *mut PixelBGRA;
            for pixel in data {
                body_ptr.write(pixel.clone());
                body_ptr = body_ptr.add(1);
            }

        };
        BGRA {
            data: buf.into_boxed_slice()
        }
    }
}
//boilerplate
impl Default for PixelBGRA {
    fn default() -> Self {
        PixelBGRA {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

impl From<(u8,u8,u8,u8)> for PixelBGRA {
    fn from((r,g,b,a): (u8,u8,u8,u8)) -> Self {
        PixelBGRA {
            r,g,b,a
        }
    }
}

impl From<PixelBGRA> for (u8,u8,u8,u8) {
    fn from(p: PixelBGRA) -> Self {
        (p.r,p.g,p.b,p.a)
    }
}

impl Default for BGRA {
    fn default() -> Self {
        BGRA {
            data: Box::new([])
        }
    }
}

impl From<Box<[u8]>> for BGRA {
    fn from(data: Box<[u8]>) -> Self {
        BGRA {
            data
        }
    }
}

impl From<BGRA> for Box<[u8]> {
    fn from(bgra: BGRA) -> Self {
        bgra.data
    }
}

impl AsRef<[u8]> for BGRA {
    fn as_ref(&self) -> &[u8] {
        &self.data
    }
}

impl AsMut<[u8]> for BGRA {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}


///Unit tests.
#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use crate::{PixelBGRA, BGRA};

    #[test]
    fn it_works() {

        let file = BGRA::new(1, 1, &[PixelBGRA {
            r: 255,
            g: 127,
            b: 3,
            a: 0
        }]);
        let data = file.into_data();
        let mut file = File::create(Path::new("test.tga")).unwrap();
        file.write(&data).unwrap();
    }
    #[test]
    fn increasing() {
        let mut pixels = Vec::new();
        let width = 240;
        let height = 240;
        for x in 0..width {
            for y in 0..height {
                pixels.push(PixelBGRA {
                    r: x,
                    g: y,
                    b: 0,
                    a: 0,
                })
            }
        }
        let rgba = BGRA::new(width as u16, height as u16, &pixels);
        let data = rgba.into_data();
        let mut file = File::create(Path::new("increasing.tga")).unwrap();
        file.write(&data).unwrap();
    }
    #[test]
    fn big() {
        let mut pixels = Vec::new();
        let width = 5000;
        let height = 5000;
        for x in 0..width {
            for y in 0..height {
                pixels.push(PixelBGRA {
                    r: (x * 255 / width) as u8,
                    g: (y * 255 / height) as u8,
                    b: (y * 255 / height) as u8,
                    a: 255,
                })
            }
        }
        let rgba = BGRA::new(width as u16, height as u16, &pixels);
        let data = rgba.into_data();
        let mut file = File::create(Path::new("big.tga")).unwrap();
        file.write(&data).unwrap();
    }
}




